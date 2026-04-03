// Numera Constants Database
// All physical and mathematical constants organized by category.

/// A physical/mathematical constant
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Constant {
    pub name: &'static str,
    #[allow(dead_code)]
    pub value: &'static str,
    pub unit: &'static str,
    #[allow(dead_code)]
    pub category: &'static str,
}

/// Get all built-in physical and mathematical constants.
///
/// # Examples
///
/// ```
/// use numera::constants::all_constants;
///
/// let consts = all_constants();
/// assert!(consts.len() > 40);
/// assert!(consts.iter().any(|c| c.name.contains("Speed of Light")));
/// ```
#[allow(dead_code)]
pub fn all_constants() -> Vec<Constant> {
    vec![
        // === General Physics ===
        Constant {
            name: "Characteristic Impedance of Vacuum",
            value: "376.730313461",
            unit: "Ω",
            category: "General Physics",
        },
        Constant {
            name: "Electric Constant",
            value: "8.854187817e-12",
            unit: "F/m",
            category: "General Physics",
        },
        Constant {
            name: "Magnetic Constant",
            value: "1.256637061e-6",
            unit: "N/A²",
            category: "General Physics",
        },
        Constant {
            name: "Gravitation Constant",
            value: "6.67428e-11",
            unit: "m³/(kg·s²)",
            category: "General Physics",
        },
        Constant {
            name: "Planck's Constant",
            value: "6.62606896e-34",
            unit: "J·s",
            category: "General Physics",
        },
        Constant {
            name: "Dirac's Constant",
            value: "1.054571628e-34",
            unit: "J·s",
            category: "General Physics",
        },
        Constant {
            name: "Speed of Light in Vacuum",
            value: "299792458",
            unit: "m/s",
            category: "General Physics",
        },
        // === Electromagnetic ===
        Constant {
            name: "Bohr-Procopiu Magneton",
            value: "927.400949e-26",
            unit: "J/T",
            category: "Electromagnetic",
        },
        Constant {
            name: "Conductance Quantum",
            value: "7.748091733e-5",
            unit: "S",
            category: "Electromagnetic",
        },
        Constant {
            name: "Coulomb's Constant",
            value: "8.987742438e9",
            unit: "N·m²/C²",
            category: "Electromagnetic",
        },
        Constant {
            name: "Elementary Charge",
            value: "1.60217653e-19",
            unit: "C",
            category: "Electromagnetic",
        },
        Constant {
            name: "Josephson Constant",
            value: "483597.879e9",
            unit: "Hz/V",
            category: "Electromagnetic",
        },
        Constant {
            name: "Magnetic Flux Quantum",
            value: "2.06783372e-15",
            unit: "Wb",
            category: "Electromagnetic",
        },
        Constant {
            name: "Nuclear Magneton",
            value: "5.05078343e-27",
            unit: "J/T",
            category: "Electromagnetic",
        },
        Constant {
            name: "Resistance Quantum",
            value: "12906.403725",
            unit: "Ω",
            category: "Electromagnetic",
        },
        Constant {
            name: "von Klitzing Constant",
            value: "25812.807449",
            unit: "Ω",
            category: "Electromagnetic",
        },
        // === Atomic & Nuclear ===
        Constant {
            name: "Bohr Radius",
            value: "0.5291772108e-10",
            unit: "m",
            category: "Atomic & Nuclear",
        },
        Constant {
            name: "Fermi Coupling Constant",
            value: "1.16639e-5",
            unit: "GeV⁻²",
            category: "Atomic & Nuclear",
        },
        Constant {
            name: "Fine-structure Constant",
            value: "7.297352568e-3",
            unit: "",
            category: "Atomic & Nuclear",
        },
        Constant {
            name: "Hartree Energy",
            value: "4.35974417e-18",
            unit: "J",
            category: "Atomic & Nuclear",
        },
        Constant {
            name: "Quantum of Circulation",
            value: "3.636947550e-4",
            unit: "m²/s",
            category: "Atomic & Nuclear",
        },
        Constant {
            name: "Rydberg Constant",
            value: "10973731.568525",
            unit: "m⁻¹",
            category: "Atomic & Nuclear",
        },
        Constant {
            name: "Thomson Cross Section",
            value: "0.665245873e-28",
            unit: "m²",
            category: "Atomic & Nuclear",
        },
        Constant {
            name: "Weak Mixing Angle",
            value: "0.22215",
            unit: "",
            category: "Atomic & Nuclear",
        },
        // === Physico-chemical ===
        Constant {
            name: "Atomic Mass Unit",
            value: "1.66053886e-27",
            unit: "kg",
            category: "Physico-chemical",
        },
        Constant {
            name: "Avogadro's Number",
            value: "6.0221415e23",
            unit: "",
            category: "Physico-chemical",
        },
        Constant {
            name: "Boltzmann Constant",
            value: "1.3806505e-23",
            unit: "J/K",
            category: "Physico-chemical",
        },
        Constant {
            name: "Faraday Constant",
            value: "96485.3383",
            unit: "C/mol",
            category: "Physico-chemical",
        },
        Constant {
            name: "First Radiation Constant",
            value: "3.74177138e-16",
            unit: "W·m²",
            category: "Physico-chemical",
        },
        Constant {
            name: "Loschmidt Constant",
            value: "2.6867773e25",
            unit: "m⁻³",
            category: "Physico-chemical",
        },
        Constant {
            name: "Gas Constant",
            value: "8.314472",
            unit: "J/(K·mol)",
            category: "Physico-chemical",
        },
        Constant {
            name: "Molar Planck Constant",
            value: "3.990312716e-10",
            unit: "J·s/mol",
            category: "Physico-chemical",
        },
        Constant {
            name: "Second Radiation Constant",
            value: "1.4387752e-2",
            unit: "m·K",
            category: "Physico-chemical",
        },
        Constant {
            name: "Stefan-Boltzmann Constant",
            value: "5.670400e-8",
            unit: "W/(m²·K⁴)",
            category: "Physico-chemical",
        },
        // === Astronomy ===
        Constant {
            name: "Astronomical Unit",
            value: "149597870691",
            unit: "m",
            category: "Astronomy",
        },
        Constant {
            name: "Light Year",
            value: "9.4607304725808e15",
            unit: "m",
            category: "Astronomy",
        },
        Constant {
            name: "Parsec",
            value: "3.08567802e16",
            unit: "m",
            category: "Astronomy",
        },
        Constant {
            name: "Sidereal Year",
            value: "365.2564",
            unit: "days",
            category: "Astronomy",
        },
        Constant {
            name: "Tropical Year",
            value: "365.2422",
            unit: "days",
            category: "Astronomy",
        },
        Constant {
            name: "Gregorian Year",
            value: "365.2425",
            unit: "days",
            category: "Astronomy",
        },
        Constant {
            name: "Earth Mass",
            value: "5.9736e24",
            unit: "kg",
            category: "Astronomy",
        },
        Constant {
            name: "Sun Mass",
            value: "1.9891e30",
            unit: "kg",
            category: "Astronomy",
        },
        Constant {
            name: "Mean Earth Radius",
            value: "6371000",
            unit: "m",
            category: "Astronomy",
        },
        Constant {
            name: "Sun Radius",
            value: "6.96265e8",
            unit: "m",
            category: "Astronomy",
        },
        Constant {
            name: "Sun Luminosity",
            value: "3.827e26",
            unit: "W",
            category: "Astronomy",
        },
    ]
}

/// Get sorted unique category names
pub fn categories() -> Vec<&'static str> {
    let mut cats: Vec<&str> = all_constants().iter().map(|c| c.category).collect();
    cats.sort_unstable();
    cats.dedup();
    cats
}

/// Get all constants belonging to a category
pub fn constants_in_category(category: &str) -> Vec<Constant> {
    all_constants()
        .into_iter()
        .filter(|c| c.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_constants_non_empty() {
        let constants = all_constants();
        assert!(
            constants.len() >= 44,
            "Expected at least 44 constants, got {}",
            constants.len()
        );
    }

    #[test]
    fn test_all_constants_have_values() {
        for c in all_constants() {
            assert!(!c.value.is_empty(), "Constant '{}' has empty value", c.name);
            assert!(!c.name.is_empty(), "Found constant with empty name");
            // Value should be parseable as a number
            let cleaned = c
                .value
                .replace("e", "E")
                .replace("-", "")
                .replace(".", "")
                .replace("+", "");
            assert!(
                cleaned.chars().all(|ch| ch.is_ascii_digit() || ch == 'E'),
                "Constant '{}' has unparseable value '{}'",
                c.name,
                c.value
            );
        }
    }

    #[test]
    fn test_categories_complete() {
        let cats = categories();
        assert!(cats.contains(&"General Physics"));
        assert!(cats.contains(&"Electromagnetic"));
        assert!(cats.contains(&"Atomic & Nuclear"));
        assert!(cats.contains(&"Physico-chemical"));
        assert!(cats.contains(&"Astronomy"));
        assert_eq!(cats.len(), 5);
    }

    #[test]
    fn test_constants_in_category() {
        let gp = constants_in_category("General Physics");
        assert!(gp.len() >= 7);
        // Speed of light should exist
        assert!(gp.iter().any(|c| c.name.contains("Speed of Light")));
    }

    #[test]
    fn test_well_known_constants() {
        let all = all_constants();
        // Speed of light
        let c = all
            .iter()
            .find(|c| c.name.contains("Speed of Light"))
            .unwrap();
        assert!(c.value.starts_with("299792458"));
        // Avogadro
        let na = all.iter().find(|c| c.name.contains("Avogadro")).unwrap();
        assert!(na.value.contains("6.022"));
        // Planck
        let h = all
            .iter()
            .find(|c| {
                c.name.contains("Planck") && !c.name.contains("Molar") && !c.name.contains("Dirac")
            })
            .unwrap();
        assert!(h.value.contains("6.626"));
    }
}
