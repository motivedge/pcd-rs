use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use crate::utils::load_meta;
use crate::{DynReader, DynRecord, Field, PcdDeserialize, PcdMeta, Reader};

use anyhow::Result;

pub fn load_pcd_meta<P>(path: P) -> Result<PcdMeta>
where
    P: AsRef<Path>,
{
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);

    let mut line_count = 0;
    load_meta(&mut reader, &mut line_count)
}

/// Read file as DynRecord, and put all fields with its value into
/// the hashmap.
/// This won't ask us to use pre-defined `struct`. All data will
/// read dynamicly and put into the `HashMap`, where key is the field name
/// and value is the data read from file.
/// BUT this function will load all data at once
pub fn load_dyn_records<P>(path: P) -> Result<Vec<HashMap<String, Field>>>
where
    P: AsRef<Path>,
{
    let mut reader = DynReader::open(path)?;
    let meta = reader.meta().to_owned();

    let mut res: Vec<HashMap<String, Field>> =
        Vec::with_capacity(meta.num_points.try_into().unwrap());
    while let Some(next_pt) = reader.next() {
        let pt = next_pt?;
        let hm_record: HashMap<String, Field> = meta
            .field_defs
            .fields
            .iter()
            .zip(pt.0.into_iter())
            .map(|(field, record)| (field.name.to_string(), record))
            .collect();
        res.push(hm_record);
    }
    Ok(res)
}

/// Similar with `load_dyn_records`. This is iterator struct.
pub struct HMRecords<T, R: Read> {
    pub reader: Reader<T, R>,
}

impl HMRecords<DynRecord, BufReader<File>> {
    pub fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path> + Clone,
    {
        let reader = DynReader::open(path)?;
        Ok(HMRecords { reader })
    }
}

impl<R> Iterator for HMRecords<DynRecord, R>
where
    R: Read + BufRead,
{
    type Item = Result<HashMap<String, Field>>;

    fn next(&mut self) -> Option<Self::Item> {
        let meta = self.reader.meta().to_owned();
        if let Some(next_pt) = self.reader.next() {
            let pt = match next_pt {
                Ok(pt) => pt,
                Err(e) => return Some(Err(e)),
            };
            let hm_record: HashMap<String, Field> = meta
                .field_defs
                .fields
                .iter()
                .zip(pt.0.into_iter())
                .map(|(field, record)| (field.name.to_string(), record))
                .collect();
            Some(Ok(hm_record))
        } else {
            None
        }
    }
}
