use anyhow::Result;
use crate::model::protein::{Protein, Chain, Residue, Atom, SecondaryStructure};
use crate::model::secondary::assign_from_pdb_file;

/// Load a protein structure from a PDB or mmCIF file
pub fn load_structure(path: &str) -> Result<Protein> {
    let (pdb, _errors) = pdbtbx::open(path)
        .map_err(|e| anyhow::anyhow!("Failed to open structure file: {:?}", e))?;

    let mut chains = Vec::new();

    for chain in pdb.chains() {
        let mut residues = Vec::new();
        for residue in chain.residues() {
            let mut atoms = Vec::new();
            for atom in residue.atoms() {
                atoms.push(Atom {
                    name: atom.name().to_string(),
                    element: atom.element().map(|e| format!("{:?}", e)).unwrap_or_default(),
                    x: atom.x(),
                    y: atom.y(),
                    z: atom.z(),
                    b_factor: atom.b_factor(),
                    is_ca: atom.name() == "CA",
                });
            }
            residues.push(Residue {
                name: residue.name().unwrap_or("UNK").to_string(),
                seq_num: residue.serial_number() as i32,
                atoms,
                secondary_structure: SecondaryStructure::Coil,
            });
        }
        chains.push(Chain {
            id: chain.id().to_string(),
            residues,
        });
    }

    let name = pdb.identifier.as_deref().unwrap_or("Unknown").to_string();

    let mut protein = Protein { name, chains };

    // Assign secondary structure from HELIX/SHEET records in the PDB file
    assign_from_pdb_file(&mut protein, path);

    Ok(protein)
}
