use std::fs::File;
use std::io::Read;
use std::mem::take;
use std::path::PathBuf;

use crate::cover::{parse_cover_record, Cover};
use crate::csv::{CsvParser, Sep};
use crate::header::{parse_header, Header, HeaderParseError};
use crate::record::Record;

/// A FEC file, the core data structure of this crate.
///
/// You create a FecFile from a stream of bytes (e.g. a file, an HTTP stream,
/// a python callback function, or some other custom source).
///
/// All methods are lazy and streaming, so nothing is read from the source
/// until you call a method that requires it.
pub struct FecFile {
    /// The source of raw bytes
    reader: Option<Box<dyn Read>>,
    header: Option<Header>,
    cover: Option<Cover>,
    sep: Option<Sep>,
    /// After reading the header, this contains the CSV reader
    /// that will be used to read the rest of the file.
    csv_parser: Option<CsvParser<Box<dyn Read>>>,
}

impl FecFile {
    pub fn from_reader(reader: Box<dyn Read>) -> Self {
        Self {
            reader: Some(reader),
            header: None,
            cover: None,
            sep: None,
            csv_parser: None,
        }
    }

    pub fn from_path(path: &PathBuf) -> Self {
        let file = File::open(path).expect("Couldn't open file");
        Self::from_reader(Box::new(file))
    }

    pub fn get_header(&mut self) -> Result<&Header, HeaderParseError> {
        self.parse_header()?;
        Ok(self.header.as_ref().expect("header should be set"))
    }

    // TODO: should this not return a reference?
    pub fn get_cover(&mut self) -> Result<&Cover, String> {
        self.parse_cover()?;
        Ok(self.cover.as_ref().expect("cover should be set"))
    }

    pub fn next_record(&mut self) -> Result<Option<Result<Record, String>>, String> {
        self.parse_cover()?;
        let p = self.csv_parser.as_mut().expect("No row parser");
        Ok(p.next_record())
    }

    fn parse_header(&mut self) -> Result<(), HeaderParseError> {
        if self.header.is_some() {
            return Ok(());
        }
        if self.reader.is_none() {
            panic!("No reader")
        }
        let header_parsing = parse_header(self.reader.as_mut().unwrap())?;
        self.header = Some(header_parsing.header.clone());
        self.sep = Some(header_parsing.sep.clone());
        Ok(())
    }

    fn parse_cover(&mut self) -> Result<(), String> {
        if self.cover.is_some() {
            return Ok(());
        }
        self.make_csv_parser()?;
        let p = self.csv_parser.as_mut().expect("No row parser");
        let record = match p.next_record() {
            None => return Err("No cover line".to_string()),
            Some(Ok(record)) => record,
            Some(Err(e)) => return Err(e),
        };
        self.cover = Some(parse_cover_record(&record)?);
        Ok(())
    }

    fn make_csv_parser(&mut self) -> Result<(), String> {
        if self.csv_parser.is_some() {
            return Ok(());
        }
        self.parse_header().map_err(|e| e.to_string())?;
        let header = self.header.as_ref().expect("No header");
        let sep = self.sep.as_ref().expect("No sep");
        if self.csv_parser.is_none() {
            // Hand off the reader ownership to the row parser.
            let reader = take(&mut self.reader).ok_or("No reader")?;
            self.csv_parser = Some(CsvParser::new(reader, header.fec_version.clone(), &sep));
        }
        Ok(())
    }
}
