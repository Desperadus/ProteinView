use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::model::protein::{Protein, SecondaryStructure};

/// A secondary structure range parsed from PDB HELIX/SHEET records.
#[derive(Debug, Clone)]
pub struct SSRange {
    pub chain_id: char,
    pub start_seq: i32,
    pub end_seq: i32,
    pub ss_type: SecondaryStructure,
}

/// Parse HELIX and SHEET records from a PDB file and return a list of SSRange entries.
///
/// PDB format column positions (0-indexed):
///   HELIX: initChainID=col 19, initSeqNum=cols 21..25, endChainID=col 31, endSeqNum=cols 33..37
///   SHEET: initChainID=col 21, initSeqNum=cols 22..26, endChainID=col 32, endSeqNum=cols 33..37
fn parse_ss_records(file_path: &str) -> Vec<SSRange> {
    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    let reader = BufReader::new(file);
    let mut ranges = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        if line.starts_with("HELIX ") {
            if line.len() < 38 {
                continue;
            }
            // initChainID at col 19
            let init_chain = line.as_bytes()[19] as char;
            // initSeqNum at cols 21..25 (inclusive, i.e. bytes 21..=24)
            let init_seq_str = &line[21..25];
            // endChainID at col 31
            let end_chain = line.as_bytes()[31] as char;
            // endSeqNum at cols 33..37 (inclusive, i.e. bytes 33..=36)
            let end_seq_str = &line[33..37];

            let init_seq: i32 = match init_seq_str.trim().parse() {
                Ok(n) => n,
                Err(_) => continue,
            };
            let end_seq: i32 = match end_seq_str.trim().parse() {
                Ok(n) => n,
                Err(_) => continue,
            };

            // Both chain IDs should match for a valid helix range.
            // We use the initChainID as the canonical chain.
            if init_chain == end_chain {
                ranges.push(SSRange {
                    chain_id: init_chain,
                    start_seq: init_seq,
                    end_seq: end_seq,
                    ss_type: SecondaryStructure::Helix,
                });
            }
        } else if line.starts_with("SHEET ") {
            if line.len() < 38 {
                continue;
            }
            // initChainID at col 21
            let init_chain = line.as_bytes()[21] as char;
            // initSeqNum at cols 22..26 (inclusive, i.e. bytes 22..=25)
            let init_seq_str = &line[22..26];
            // endChainID at col 32
            let end_chain = line.as_bytes()[32] as char;
            // endSeqNum at cols 33..37 (inclusive, i.e. bytes 33..=36)
            let end_seq_str = &line[33..37];

            let init_seq: i32 = match init_seq_str.trim().parse() {
                Ok(n) => n,
                Err(_) => continue,
            };
            let end_seq: i32 = match end_seq_str.trim().parse() {
                Ok(n) => n,
                Err(_) => continue,
            };

            if init_chain == end_chain {
                ranges.push(SSRange {
                    chain_id: init_chain,
                    start_seq: init_seq,
                    end_seq: end_seq,
                    ss_type: SecondaryStructure::Sheet,
                });
            }
        }
    }

    ranges
}

/// Parse HELIX/SHEET records from the given PDB file and assign
/// secondary structure to all matching residues in the protein.
/// Residues not covered by any HELIX or SHEET record remain as Coil.
pub fn assign_from_pdb_file(protein: &mut Protein, file_path: &str) {
    let ranges = parse_ss_records(file_path);
    if ranges.is_empty() {
        return;
    }

    for chain in &mut protein.chains {
        // The chain id in our model is a String; PDB records use a single char.
        let chain_char = chain.id.chars().next().unwrap_or(' ');
        for residue in &mut chain.residues {
            for range in &ranges {
                if range.chain_id == chain_char
                    && residue.seq_num >= range.start_seq
                    && residue.seq_num <= range.end_seq
                {
                    residue.secondary_structure = range.ss_type;
                    break; // first matching range wins
                }
            }
        }
    }
}
