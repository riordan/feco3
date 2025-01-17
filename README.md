# FECo3

A [.FEC](https://www.fec.gov/introduction-campaign-finance/data-tutorials/)
file parser in rust, with python bindings. The rust is intended to
be extendable, easy to maintain, and performant. The python is intended to
be easy to use, with type hints, possible to extend,
integrate with the rest of the python data ecosystem.

Still in alpha.

## Links

- [Python docs](https://nickcrews.github.io/feco3/), if you want to use the Python API.
- [Rust docs](https://docs.rs/feco3), if you want to use the Rust API.
- [.fec file format reference](https://github.com/NickCrews/feco3/wiki/.fec-File-Format)
  if you want to know more about the .fec file format or are interested in writing
  your own parser or improving this one.

## Example Python

```python
import pyarrow as pa
import feco3

# ruff: noqa: E501

# You can supply a URL or a path to a file.
# Possibly in the future we'll support reading from a file-like object.
src = "https://docquery.fec.gov/dcdev/posted/1002596.fec"
# src = "path/to/file.fec"
# src = pathlib.Path("path/to/file.fec")

# The straightforward way is to just parse to a directory of files,
# one file for each itemization type, eg "csvs/SA11AI.csv", etc
feco3.FecFile(src).to_csvs("csvs/")
feco3.FecFile(src).to_parquets("parquets/")

# Or, you can look at the file at a lower level.
# This doesn't actually read or parse any data yet
fec = feco3.FecFile(src)
print(fec)
# FecFile(src='https://docquery.fec.gov/dcdev/posted/1002596.fec')

# Only when we access something do we actually start parsing.
# Still, we only parse as far as we need to, so this is quite fast.
# This is useful, for example, if you only need the header or cover,
# or if you only want to look at the itemizations in certain forms.
print(fec.header)
print(fec.cover)
# Header(fec_version='8.1', software_name='NetFile', software_version='199199', report_id=None, report_number='0')
# Cover(form_type='F3N', filer_committee_id='C00479188')

# Iterate through the itemizations in the file in batches of pyarrow RecordBatches.
# By iterating, this keeps us from having to load the entire file into memory.
# By using pyarrow, we can avoid copying the underlying data from Rust to Python.
# It integrates well with the rest of the Python data ecosystem, for example
# it's easy to convert to a pandas DataFrames.
batcher = feco3.PyarrowBatcher(fec, max_batch_size=1024 * 1024)
for batch in batcher:
    # The record code for this kind of itemizations, eg. 'SA11AI'
    assert isinstance(batch.code, str)
    # A pyarrow RecordBatch of the itemizations
    assert isinstance(batch.records, pa.RecordBatch)
    df = batch.records.to_pandas()
    print(batch.code)
    print(df.head(3))
# SA15
#   filer_committee_id_number transaction_id back_reference_tran_id_number back_reference_sched_name  ... conduit_zip_code memo_code memo_text_description reference_code
# 0                 C00479188        INCA994                                                          ...
# 1                 C00479188        INCA992                                                          ...
# 2                 C00479188        INCA993                                                          ...

# [3 rows x 44 columns]
# TEXT
#   filer_committee_id_number transaction_id_number back_reference_tran_id_number back_reference_sched_form_name            text
# 0                 C00479188              TPAYC760                       PAYC760                          SC/10  PERSONAL FUNDS
# SC/10
#   filer_committee_id_number transaction_id_number receipt_line_number entity_type  ... lender_candidate_state lender_candidate_district memo_code memo_text_description
# 0                 C00479188               PAYC760                 13B         CAN  ...

# [1 rows x 37 columns]                       ...

```



## Related projects

Please open an issue or PR if you'd like to add or edit this list.

- [FECfile](https://github.com/esonderegger/fecfile)
  Fairly well maintained parser in python
- [FastFEC](https://github.com/washingtonpost/FastFEC)
  A FEC file parser in C.
- [fec-loader](https://github.com/PublicI/fec-loader)
  Node.js tools and CLI to discover, convert and load raw FEC filings into a database.
- [Fech](https://github.com/dwillis/Fech)
  Ruby downloader and parser. Moderately recently maintained?
- [fech-sources](https://github.com/dwillis/fech-sources)
  Schema definitions for the various line codes. Used by Fech and some other parsers.
- [nyt-pyfec](https://github.com/newsdev/nyt-pyfec)
  Old, unmaintained python parser
- [fec2json](https://github.com/newsdev/fec2json)
  Not complete parser written in python
