use std::{collections::HashMap, io};

use bytes::{Buf, Bytes};

use super::get_encoding;
use crate::{data_container::compression_header::TagEncodingMap, reader::num::get_itf8};

pub fn get_tag_encoding_map(src: &mut Bytes) -> io::Result<TagEncodingMap> {
    let data_len = get_itf8(src).and_then(|n| {
        usize::try_from(n).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    })?;

    if src.remaining() < data_len {
        return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
    }

    let mut buf = src.split_to(data_len);

    let map_len = get_itf8(&mut buf)?;

    let mut map = HashMap::with_capacity(map_len as usize);

    for _ in 0..map_len {
        let key = get_itf8(&mut buf)?;
        let encoding = get_encoding(&mut buf)?;
        map.insert(key, encoding);
    }

    Ok(TagEncodingMap::from(map))
}
