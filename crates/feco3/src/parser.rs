use std::io::Read;
use std::mem::take;

use crate::form::{lookup_schema, FieldSchema, FormLine, ValueType};
use crate::header::{parse_header, HeaderParseError, HeaderParsing};
// use csv::Reader;
use csv::ReaderBuilder;

pub struct Parser<R: Read> {
    /// If parsed yet, contains the header
    pub header_parsing: Option<HeaderParsing>,
    /// The source of raw bytes
    reader: Option<R>,
    /// After reading the header, this contains the CSV reader
    /// that will be used to read the rest of the file.
    row_parser: Option<RowsParser<R>>,
}

impl<R: Read> Parser<R> {
    pub fn from_reader(reader: R) -> Self {
        Self {
            reader: Some(reader),
            header_parsing: None,
            row_parser: None,
        }
    }

    pub fn parse_header(&mut self) -> Result<&HeaderParsing, HeaderParseError> {
        if self.reader.is_none() {
            panic!("No reader")
        }
        let header_parsing = parse_header(self.reader.as_mut().unwrap())?;
        self.header_parsing = Some(header_parsing);
        let result = self.header_parsing.as_ref().unwrap();
        Ok(result)
    }

    pub fn next_line(&mut self) -> Result<Option<Result<FormLine, String>>, String> {
        if self.row_parser.is_none() {
            // Hand off the reader ownership to the row parser.
            let reader = take(&mut self.reader).ok_or("No reader")?;
            self.row_parser = Some(RowsParser::new(
                reader,
                self.header_parsing.as_ref().unwrap().header.version.clone(),
                self.header_parsing.as_ref().unwrap().uses_ascii28,
            ));
        }
        let rp = self.row_parser.as_mut().ok_or("No row parser")?;
        let line = rp.next_line();
        Ok(line)
    }
}

struct RowsParser<R: Read> {
    /// The version of the FEC file format
    version: String,
    records: csv::ByteRecordsIntoIter<R>,
}

impl<R: Read> RowsParser<R> {
    fn new(src: R, version: String, use_ascii28: bool) -> Self {
        let delim = if use_ascii28 { b'\x1c' } else { b',' };
        let reader = ReaderBuilder::new()
            .delimiter(delim)
            .has_headers(false)
            .flexible(true)
            .from_reader(src);
        Self {
            version,
            records: reader.into_byte_records(),
        }
    }

    fn next_line(&mut self) -> Option<Result<FormLine, String>> {
        let raw_record = self.records.next();
        log::debug!("raw_record: {:?}", raw_record);
        let record_or_error = raw_record?.map_err(|e| e.to_string());
        let record = match record_or_error {
            Ok(record) => record,
            Err(e) => return Some(Err(e)),
        };
        Some(self.parse_csv_record(record))
    }

    fn parse_csv_record(&self, record: csv::ByteRecord) -> Result<FormLine, String> {
        let mut record_fields = record.iter();
        let form_name = match record_fields.next() {
            Some(form_name) => form_name,
            None => return Err("No form name".to_string()),
        };
        let form_name_str = String::from_utf8(form_name.to_vec()).map_err(|e| e.to_string())?;
        let form_schema = lookup_schema(&self.version, &form_name_str)?;
        let mut schema_fields = form_schema.fields.iter();
        let mut fields = Vec::new();
        for raw_value in record_fields {
            fields.push(parse_raw_field_val(raw_value, schema_fields.next())?);
        }
        let extra_schema_fields = schema_fields.count();
        if extra_schema_fields > 0 {
            log::error!("extra_schema_fields: {}", extra_schema_fields);
        }
        Ok(FormLine {
            form_schema: form_schema.clone(),
            fields,
        })
    }
}

fn parse_raw_field_val(
    raw_value: &[u8],
    field_schema: Option<&FieldSchema>,
) -> Result<crate::form::Field, String> {
    let s = String::from_utf8_lossy(raw_value).to_string();
    let default_field_schema = FieldSchema {
        name: "extra".to_string(),
        typ: ValueType::String,
    };
    let field_schema = field_schema.unwrap_or(&default_field_schema);
    let parsed_val = match field_schema.typ {
        crate::form::ValueType::String => crate::form::Value::String(s),
        crate::form::ValueType::Integer => {
            let i = s.parse::<i64>().map_err(|e| e.to_string())?;
            crate::form::Value::Integer(i)
        }
        crate::form::ValueType::Float => {
            let f = s.parse::<f64>().map_err(|e| e.to_string())?;
            crate::form::Value::Float(f)
        }
        crate::form::ValueType::Date => crate::form::Value::Date(s),
        crate::form::ValueType::Boolean => {
            let b = s.parse::<bool>().map_err(|e| e.to_string())?;
            crate::form::Value::Boolean(b)
        }
    };
    Ok(crate::form::Field {
        name: field_schema.name.clone(),
        value: parsed_val,
    })
}
