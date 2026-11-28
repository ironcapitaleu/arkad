//! # Sample Notes Archive
//!
//! Provides [`NotesZipBuilder`], which writes a small Notes ZIP in memory, and the members of a
//! valid sample Notes file.
//!
//! The builder writes each byte of the ZIP itself, so a test can damage one field, such as the
//! CRC-32 or the compression method, and keep the rest of the archive valid. The sample members
//! hold a few rows each. One field of `dim.tsv` is quoted and holds a tab.

use std::io::Write;

use flate2::Compression;
use flate2::Crc;
use flate2::write::DeflateEncoder;

/// The compression method of a member, as the ZIP index records it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleCompression {
    /// Method 0: the bytes as they are.
    Stored,
    /// Method 8: deflate.
    Deflated,
}

/// One member of a sample ZIP, with the fields of the ZIP index that a test can override.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleMember {
    /// The name of the member.
    pub name: String,
    /// The unzipped bytes of the member.
    pub content: Vec<u8>,
    /// The compression method of the member.
    pub compression: SampleCompression,
    /// The CRC-32 the ZIP index records, instead of the CRC-32 of `content`.
    pub crc32: Option<u32>,
    /// The unzipped size the ZIP index records, instead of the size of `content`.
    pub unzipped_size: Option<u32>,
}

impl SampleMember {
    /// Creates a deflated [`SampleMember`] whose ZIP index matches its content.
    #[must_use]
    pub fn new(name: &str, content: &[u8]) -> Self {
        Self {
            name: name.to_owned(),
            content: content.to_vec(),
            compression: SampleCompression::Deflated,
            crc32: None,
            unzipped_size: None,
        }
    }
}

/// Writes the members of a sample ZIP, in order, with no ZIP64 field and no data descriptor.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NotesZipBuilder {
    members: Vec<SampleMember>,
}

impl NotesZipBuilder {
    /// Creates a [`NotesZipBuilder`] with the members of [`sample_members`].
    #[must_use]
    pub fn sample() -> Self {
        Self {
            members: sample_members(),
        }
    }

    /// Creates a [`NotesZipBuilder`] with the given members.
    #[must_use]
    pub const fn with_members(members: Vec<SampleMember>) -> Self {
        Self { members }
    }

    /// Returns the members, for a test to change one of them.
    #[must_use]
    pub const fn members_mut(&mut self) -> &mut Vec<SampleMember> {
        &mut self.members
    }

    /// Returns the member with the given name, for a test to change it.
    ///
    /// # Panics
    ///
    /// Panics if no member has the name.
    pub fn member_mut(&mut self, name: &str) -> &mut SampleMember {
        self.members
            .iter_mut()
            .find(|member| member.name == name)
            .expect("Given a name of the sample members, the member should always exist")
    }

    /// Writes the ZIP: one local header and the data per member, the central directory, and the
    /// end of central directory record.
    ///
    /// # Panics
    ///
    /// Panics if a member or the archive is larger than 4 GiB.
    #[must_use]
    pub fn build(&self) -> Vec<u8> {
        let mut archive = Vec::new();
        let mut central_directory = Vec::new();
        for member in &self.members {
            let header = MemberHeader::of(member);
            let offset = to_u32(archive.len());
            archive.extend_from_slice(&0x0403_4b50_u32.to_le_bytes());
            header.write_common(&mut archive);
            archive.extend_from_slice(&0_u16.to_le_bytes());
            archive.extend_from_slice(member.name.as_bytes());
            archive.extend_from_slice(&header.data);

            central_directory.extend_from_slice(&0x0201_4b50_u32.to_le_bytes());
            central_directory.extend_from_slice(&20_u16.to_le_bytes());
            header.write_common(&mut central_directory);
            // The lengths of the extra field and the comment, the disk, and the attributes.
            central_directory.extend_from_slice(&[0; 12]);
            central_directory.extend_from_slice(&offset.to_le_bytes());
            central_directory.extend_from_slice(member.name.as_bytes());
        }
        let directory_offset = to_u32(archive.len());
        let count = u16::try_from(self.members.len())
            .expect("Given a sample with a few members, the count should always fit in 16 bits");
        archive.extend_from_slice(&central_directory);
        archive.extend_from_slice(&0x0605_4b50_u32.to_le_bytes());
        archive.extend_from_slice(&[0; 4]);
        archive.extend_from_slice(&count.to_le_bytes());
        archive.extend_from_slice(&count.to_le_bytes());
        archive.extend_from_slice(&to_u32(central_directory.len()).to_le_bytes());
        archive.extend_from_slice(&directory_offset.to_le_bytes());
        archive.extend_from_slice(&0_u16.to_le_bytes());
        archive
    }
}

/// The fields that the local header and the central directory of one member share.
struct MemberHeader {
    method: u16,
    crc32: u32,
    unzipped_size: u32,
    name_length: u16,
    data: Vec<u8>,
}

impl MemberHeader {
    fn of(member: &SampleMember) -> Self {
        let (method, data) = match member.compression {
            SampleCompression::Stored => (0, member.content.clone()),
            SampleCompression::Deflated => (8, deflate(&member.content)),
        };
        let mut crc = Crc::new();
        crc.update(&member.content);
        Self {
            method,
            crc32: member.crc32.unwrap_or_else(|| crc.sum()),
            unzipped_size: member
                .unzipped_size
                .unwrap_or_else(|| to_u32(member.content.len())),
            name_length: u16::try_from(member.name.len())
                .expect("Given a short sample name, its length should always fit in 16 bits"),
            data,
        }
    }

    /// Writes the fields from "version needed" to "extra field length", without the last one.
    fn write_common(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&20_u16.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.method.to_le_bytes());
        // Time 00:00:00 and date 1980-01-01.
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&0x21_u16.to_le_bytes());
        out.extend_from_slice(&self.crc32.to_le_bytes());
        out.extend_from_slice(&to_u32(self.data.len()).to_le_bytes());
        out.extend_from_slice(&self.unzipped_size.to_le_bytes());
        out.extend_from_slice(&self.name_length.to_le_bytes());
    }
}

fn deflate(content: &[u8]) -> Vec<u8> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(content)
        .expect("Given an in-memory buffer, deflating should always succeed");
    encoder
        .finish()
        .expect("Given an in-memory buffer, deflating should always succeed")
}

fn to_u32(length: usize) -> u32 {
    u32::try_from(length).expect("Given a sample smaller than 4 GiB, a size should always fit")
}

/// The header of `sub.tsv`, as the SEC writes it.
pub const SUB_HEADER: &str = "adsh\tcik\tname\tsic\tcountryba\tstprba\tcityba\tzipba\tbas1\tbas2\tbaph\tcountryma\tstprma\tcityma\tzipma\tmas1\tmas2\tcountryinc\tstprinc\tein\tformer\tchanged\tafs\twksi\tfye\tform\tperiod\tfy\tfp\tfiled\taccepted\tprevrpt\tdetail\tinstance\tnciks\taciks\tpubfloatusd\tfloatdate\tfloataxis\tfloatmems";

/// The header of `num.tsv`, as the SEC writes it.
pub const NUM_HEADER: &str = "adsh\ttag\tversion\tddate\tqtrs\tuom\tdimh\tiprx\tvalue\tfootnote\tfootlen\tdimn\tcoreg\tdurp\tdatp\tdcml";

/// The header of `txt.tsv`, as the SEC writes it.
pub const TXT_HEADER: &str = "adsh\ttag\tversion\tddate\tqtrs\tiprx\tlang\tdcml\tdurp\tdatp\tdimh\tdimn\tcoreg\tescaped\tsrclen\ttxtlen\tfootnote\tfootlen\tcontext\tvalue";

/// The header of `dim.tsv`, as the SEC writes it. The metadata member names its first column
/// `dimh`.
pub const DIM_HEADER: &str = "dimhash\tsegments\tsegt";

/// The header of `tag.tsv`, as the SEC writes it.
pub const TAG_HEADER: &str = "tag\tversion\tcustom\tabstract\tdatatype\tiord\tcrdr\ttlabel\tdoc";

/// The header of `pre.tsv`, as the SEC writes it.
pub const PRE_HEADER: &str =
    "adsh\treport\tline\tstmt\tinpth\ttag\tversion\tprole\tplabel\tnegating";

/// The header of `cal.tsv`, as the SEC writes it.
pub const CAL_HEADER: &str = "adsh\tgrp\tarc\tnegative\tptag\tpversion\tctag\tcversion";

/// The header of `ren.tsv`, as the SEC writes it.
pub const REN_HEADER: &str = "adsh\treport\trfile\tmenucat\tshortname\tlongname\troleuri\tparentroleuri\tparentreport\tultparentrpt";

/// The eight tables and their headers, in the order of the metadata member.
pub const SAMPLE_HEADERS: [(&str, &str); 8] = [
    ("sub.tsv", SUB_HEADER),
    ("tag.tsv", TAG_HEADER),
    ("dim.tsv", DIM_HEADER),
    ("num.tsv", NUM_HEADER),
    ("txt.tsv", TXT_HEADER),
    ("ren.tsv", REN_HEADER),
    ("pre.tsv", PRE_HEADER),
    ("cal.tsv", CAL_HEADER),
];

/// The `dim.tsv` of the sample: a header and two rows. The second row quotes a field that holds a
/// tab.
pub const SAMPLE_DIM_TSV: &str =
    "dimhash\tsegments\tsegt\n0x00000000\t\t0\n0x1a2b3c4d\t\"Axis=A\tB;\"\t0\n";

/// Builds the CSVW metadata member with the columns of each header, and `dimh` as the first
/// column of `dim.tsv`, as the SEC writes it.
#[must_use]
pub fn sample_metadata(headers: &[(&str, &str)]) -> String {
    let tables: Vec<String> = headers
        .iter()
        .map(|(table, header)| {
            let columns: Vec<String> = header
                .split('\t')
                .map(|column| {
                    let name = if *table == "dim.tsv" && column == "dimhash" {
                        "dimh"
                    } else {
                        column
                    };
                    format!("{{\"name\":\"{name}\",\"titles\":[]}}")
                })
                .collect();
            format!(
                "{{\"url\":\"{table}\",\"tableSchema\":{{\"columns\":[{}]}}}}",
                columns.join(",")
            )
        })
        .collect();
    format!(
        "{{\"@context\":\"http://www.w3.org/ns/csvw\",\"dialect\":{{\"delimiter\":\"\\t\"}},\"tables\":[{}]}}",
        tables.join(",")
    )
}

/// The ten members of a valid sample Notes file: the eight tables, `readme.htm`, and
/// `notes-metadata.json`. Each table holds its header, and `dim.tsv` holds [`SAMPLE_DIM_TSV`].
#[must_use]
pub fn sample_members() -> Vec<SampleMember> {
    let mut members: Vec<SampleMember> = SAMPLE_HEADERS
        .iter()
        .map(|(table, header)| {
            let content = if *table == "dim.tsv" {
                SAMPLE_DIM_TSV.to_owned()
            } else {
                format!("{header}\n")
            };
            SampleMember::new(table, content.as_bytes())
        })
        .collect();
    members.push(SampleMember::new("readme.htm", b"<html></html>"));
    members.push(SampleMember::new(
        "notes-metadata.json",
        sample_metadata(&SAMPLE_HEADERS).as_bytes(),
    ));
    members
}
