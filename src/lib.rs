use std::convert::TryInto;
use std::io::Cursor;

use rusqlite::functions::{Context, FunctionFlags};
use rusqlite::types::{ToSqlOutput, Value};

#[cfg(feature = "build_extension")]
mod ext;

fn ah(e: anyhow::Error) -> rusqlite::Error {
    rusqlite::Error::UserFunctionError(format!("{:?}", e).into())
}

fn setup(db: &rusqlite::Connection) -> anyhow::Result<()> {
    create_functions(&db)
}

fn create_functions(db: &rusqlite::Connection) -> anyhow::Result<()> {
    let flags = FunctionFlags::SQLITE_UTF8
        | FunctionFlags::SQLITE_INNOCUOUS
        | FunctionFlags::SQLITE_DETERMINISTIC;
    db.create_scalar_function("bech32m_encode", 2, flags, |ctx| {
        bech32m_encode_fn(ctx).map_err(ah)
    })?;
    db.create_scalar_function("bech32m_decode", 1, flags, |ctx| {
        bech32m_decode_fn(ctx).map_err(ah)
    })?;
    db.create_scalar_function("blob_from_hex", 1, flags, |ctx| {
        blob_from_hex_fn(ctx).map_err(ah)
    })?;
    db.create_scalar_function("chia_amount_int", 1, flags, |ctx| {
        chia_amount_int(ctx).map_err(ah)
    })?;
    db.create_scalar_function("chia_fullblock_json", 1, flags, |ctx| {
        chia_fullblock_json(ctx).map_err(ah)
    })?;
    db.create_scalar_function("sha256sum", 1, flags, |ctx| sha256sum(ctx).map_err(ah))?;
    db.create_scalar_function("zstd_decompress_blob", 1, flags, |ctx| {
        zstd_decompress_blob(ctx).map_err(ah)
    })?;
    Ok(())
}

fn bech32m_encode_fn<'a>(ctx: &Context) -> anyhow::Result<ToSqlOutput<'a>> {
    use bech32::ToBase32;
    let hrp = ctx.get::<String>(0)?;
    let data = ctx.get::<Vec<u8>>(1)?;
    let encoded = bech32::encode(&hrp, data.to_base32(), bech32::Variant::Bech32m)?;
    Ok(ToSqlOutput::Owned(Value::Text(encoded)))
}

fn bech32m_decode_fn<'a>(ctx: &Context) -> anyhow::Result<ToSqlOutput<'a>> {
    use bech32::FromBase32;
    let encoded = ctx.get::<String>(0)?;
    let (_hrp, data, _variant) = bech32::decode(&encoded)?;
    Ok(ToSqlOutput::Owned(Value::Blob(Vec::<u8>::from_base32(
        &data,
    )?)))
}

fn blob_from_hex_fn<'a>(ctx: &Context) -> anyhow::Result<ToSqlOutput<'a>> {
    let hex = ctx.get::<String>(0)?;
    let data = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<Result<Vec<u8>, std::num::ParseIntError>>()?;
    Ok(ToSqlOutput::Owned(Value::Blob(data)))
}

fn chia_amount_int<'a>(ctx: &Context) -> anyhow::Result<ToSqlOutput<'a>> {
    let blob = ctx.get::<Vec<u8>>(0)?;
    let bytes: [u8; 8] = blob.try_into().unwrap();
    let mojos = i64::from_be_bytes(bytes); // @@ i64 != u64
    Ok(ToSqlOutput::Owned(Value::Integer(mojos)))
}

fn chia_fullblock_json<'a>(ctx: &Context) -> anyhow::Result<ToSqlOutput<'a>> {
    use chia_traits::streamable::Streamable;
    let blob = ctx.get::<Vec<u8>>(0)?;
    let block = chia_protocol::FullBlock::parse::<true>(&mut Cursor::new(&blob))?;
    let json: String = serde_json::to_string(&block)?;
    Ok(ToSqlOutput::Owned(Value::Text(json)))
}

fn sha256sum<'a>(ctx: &Context) -> anyhow::Result<ToSqlOutput<'a>> {
    use sha2::Digest;
    let blob = ctx.get::<Vec<u8>>(0)?;
    let data = blob.as_slice();
    let digest = sha2::Sha256::digest(data);
    Ok(ToSqlOutput::Owned(Value::Blob(digest.to_vec())))
}

fn zstd_decompress_blob<'a>(ctx: &Context) -> anyhow::Result<ToSqlOutput<'a>> {
    let blob = ctx.get::<Vec<u8>>(0)?;
    let out = zstd::stream::decode_all(blob.as_slice())?;
    Ok(ToSqlOutput::Owned(Value::Blob(out)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use pretty_assertions::assert_eq;

    use rusqlite::Connection;

    fn open_db() -> anyhow::Result<Connection> {
        let db = Connection::open_in_memory().context("open in-memory db")?;
        setup(&db).context("setup db")?;
        Ok(db)
    }

    #[test]
    fn sanity() -> anyhow::Result<()> {
        open_db()?;
        Ok(())
    }

    #[test]
    fn bech32m_encode_works() -> anyhow::Result<()> {
        let db = open_db()?;
        assert_eq!(
            "xch1jlgazv".to_string(),
            db.query_row("select bech32m_encode('xch', x'')", [], |r| r
                .get::<usize, String>(0))?
        );
        assert_eq!(
            "xch1etlqusgk05".to_string(),
            db.query_row("select bech32m_encode('xch', x'cafe')", [], |r| r
                .get::<usize, String>(0))?
        );
        assert_eq!("xch17nmv5574vggcdxchqh8zjunt44ax05cwhcqz5e29pvf6mwc95e5s27yfa4".to_string(),
                   db.query_row("select bech32m_encode('xch', x'f4f6ca53d56211869b1705ce29726bad7a67d30ebe002a65450b13adbb05a669')",
                                [],
                                |r| r.get::<usize, String>(0))?);
        Ok(())
    }

    #[test]
    fn bech32m_decode_works() -> anyhow::Result<()> {
        let db = open_db()?;
        assert_eq!(
            "".to_string(),
            db.query_row("select hex(bech32m_decode('xch1jlgazv'))", [], |r| r
                .get::<usize, String>(0))?
        );
        assert_eq!(
            "CAFE".to_string(),
            db.query_row("select hex(bech32m_decode('xch1etlqusgk05'))", [], |r| {
                r.get::<usize, String>(0)
            })?
        );
        assert_eq!("F4F6CA53D56211869B1705CE29726BAD7A67D30EBE002A65450B13ADBB05A669".to_string(),
                   db.query_row("select hex(bech32m_decode('xch17nmv5574vggcdxchqh8zjunt44ax05cwhcqz5e29pvf6mwc95e5s27yfa4'))",
                                [],
                                |r| r.get::<usize, String>(0))?);
        Ok(())
    }

    #[test]
    fn blob_from_hex_works() -> anyhow::Result<()> {
        let db = open_db()?;
        assert_eq!(
            "CAFE".to_string(),
            db.query_row("select hex(blob_from_hex('cafe'))", [], |r| r
                .get::<usize, String>(0))?
        );
        Ok(())
    }

    #[test]
    fn chia_amount_int_works() -> anyhow::Result<()> {
        let db = open_db()?;
        assert_eq!(
            5509699999997u64,
            db.query_row("select chia_amount_int(x'00000502D3B618FD')", [], |r| r
                .get::<usize, u64>(
                0
            ))?
        );
        Ok(())
    }

    #[test]
    fn sha256sum_works() -> anyhow::Result<()> {
        let db = open_db()?;
        assert_eq!(
            "03346F0E7990DE2423A3BCA5335BF92CDC0BD14BEF2206B87C63F18A1E996C52".to_string(),
            db.query_row("select hex(sha256sum(x'cafe'))", [], |r| r
                .get::<usize, String>(0))?
        );
        Ok(())
    }

    #[test]
    fn zstd_decompress_blob_works() -> anyhow::Result<()> {
        let db = open_db()?;
        assert_eq!(
            "CAFE".to_string(),
            db.query_row(
                "select hex(zstd_decompress_blob(x'28b52ffd0458110000cafe23ae5cb0'));",
                [],
                |r| r.get::<usize, String>(0)
            )?
        );
        Ok(())
    }
}
