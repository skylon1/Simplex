use std::path::Path;
use std::fs;

#[derive(Debug)]
pub struct LinearProgram {
    /// Constraint matrix A (m x n)
    pub a: Vec<Vec<f64>>,
    /// RHS vector b (length m)
    pub b: Vec<f64>,
    /// Objective coefficients c (length n)
    pub c: Vec<f64>,
    /// Number of constraints
    pub m: usize,
    /// Number of variables
    pub n: usize,
}

#[derive(Debug)]
pub struct SimplexDict {
    pub x_n: Vec<f64>,         // Values of Non-Base Variables
    pub a: Vec<Vec<f64>>,      // Constraint matrix A (m x n)
    pub b: Vec<f64>,           // RHS vector b (len m)
    pub c: Vec<f64>,           // Objective coefficients c (len n)
    pub x_b: Vec<f64>,         // Values of Base Variables
    pub z_0: f64,              // Constant part of Objective
    pub z: f64,                // Objective Value
    pub m: usize,              // Number of Constraints
    pub n: usize,              // Number of Input Variables
    pub basis: Vec<usize>,     // Tracking of Base-Variables
    pub non_basis: Vec<usize>, // Tracking of NonBase-Variables
}

impl SimplexDict {
    pub fn from_program(input: &LinearProgram) -> Self {
        let x_n = vec![0.0; input.n];
        let x_b = input.b.clone();
        let basis: Vec<usize> = (input.n..input.n + input.m).collect();
        let non_basis: Vec<usize> = (0..input.n).collect();
        SimplexDict {
            x_n,
            a: input.a.clone(),
            b: input.b.clone(),
            c: input.c.clone(),
            x_b,
            z_0: 0.0,
            z: 0.0,
            m: input.m,
            n: input.n,
            basis,
            non_basis,
        }
    }

    pub fn solve(&mut self) -> Result<f64, String> {
        loop {
            // Abbruch: kein positiver Zielfunktionskoeffizient mehr
            if self.c.iter().all(|&ci| ci <= 0.0) {
                return Ok(self.z);
            }
            self.step()?;
        }
    }

    pub fn step(&mut self) -> Result<(), String> {
        // --- Deine Version (for-loop) ---
        // let max_id = find_maxarg(&self.c)?;
        // let mut current_min_id: Option<usize> = None;
        // let mut current_min_ratio = f64::INFINITY;
        // for i in 0..self.m {
        //     let d = self.a[i][max_id];
        //     if d > 0.0 {
        //         let ratio = self.b[i] / d;
        //         if ratio < current_min_ratio {
        //             current_min_id = Some(i);
        //             current_min_ratio = ratio;
        //         }
        //     }
        // }
        // if current_min_id == None {
        //     return Err("Unbound".to_string());
        // }

        // --- Idiomatic Rust Version ---

        // Schritt 1: Eintrittsvariable (größter positiver c-Koeffizient)
        let entering = find_maxarg(&self.c)?;
        if self.c[entering] <= 0.0 {
            return Ok(()); // Optimum erreicht
        }

        // Schritt 2: Ratio Test — Austrittsvariable bestimmen
        let leaving = (0..self.m)
            .filter(|&i| self.a[i][entering] > 0.0)
            .min_by(|&i, &j| {
                let ri = self.b[i] / self.a[i][entering];
                let rj = self.b[j] / self.a[j][entering];
                ri.partial_cmp(&rj).unwrap()
            })
            .ok_or("LP ist unbeschränkt")?;

        // Schritt 3: Pivot
        self.pivot(entering, leaving);

        Ok(())
    }

    fn pivot(&mut self, entering: usize, leaving: usize) {
        let pivot = self.a[leaving][entering];

        // Pivot-Zeile durch Pivot-Element teilen (leaving wird zu entering)
        for j in 0..self.n {
            self.a[leaving][j] /= pivot;
        }
        self.b[leaving] /= pivot;

        // Alle anderen Zeilen und die Zielfunktion aktualisieren
        for i in 0..self.m {
            if i == leaving { continue; }
            let factor = self.a[i][entering];
            for j in 0..self.n {
                self.a[i][j] -= factor * self.a[leaving][j];
            }
            self.b[i] -= factor * self.b[leaving];
        }

        // Zielfunktion updaten
        let factor = self.c[entering];
        for j in 0..self.n {
            self.c[j] -= factor * self.a[leaving][j];
        }
        self.z += factor * self.b[leaving];

        // Basis tauschen
        let temp = self.non_basis[entering];
        self.non_basis[entering] = self.basis[leaving];
        self.basis[leaving] = temp;


        // x_b aktualisieren
        self.x_b[leaving] = self.b[leaving];
    }
    fn print_result(&self){
        println!("LP Lösung:");
        println!("Zielfuktionswert: {}", self.z);
        println!("Variablen: ");
        for i in 0..self.n {
            if let Some(pos) = self.basis.iter().position(|&b| b==i){
                println!("  x{} = {}", i+1, self.x_b[pos]);
            } else {
                println!("  x{} = 0", i+1);
            }
        }
        println!("Schlupfvariablen: ");
        for i in self.n..self.m+self.n {
            if let Some(pos) = self.basis.iter().position(|&b| b==i){
                println!("  s{} = {}", i-self.n+1, self.x_b[pos]);
            }else {
                println!("  s{} = 0", i-self.n+1);
            }
        }
    }
}

impl LinearProgram {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Datei konnte nicht gelesen werden: {e}"))?;

        let mut lines = content
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty());

        // Erste Zeile: m n
        let first = lines.next().ok_or("Datei ist leer")?;
        let (m, n) = parse_m_n(first)?;

        // Nächste m Zeilen: Nebenbedingungen
        let mut a = Vec::with_capacity(m);
        let mut b = Vec::with_capacity(m);
        for i in 0..m {
            let line = lines
                .next()
                .ok_or(format!("Fehlende Nebenbedingung {}", i + 1))?;
            let (row, rhs) = parse_constraint(line, n)?;
            a.push(row);
            b.push(rhs);
        }

        // Letzte Zeile: Zielfunktionskoeffizienten
        let obj_line = lines.next().ok_or("Fehlende Zielfunktion")?;
        let c = parse_coefficients(obj_line, n)?;

        Ok(LinearProgram { a, b, c, m, n })
    }
}

fn find_maxarg(input: &[f64]) -> Result<usize, String> {
    input
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i)
        .ok_or("empty input".to_string())
}

fn parse_m_n(line: &str) -> Result<(usize, usize), String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(format!("Erste Zeile muss genau 2 Zahlen enthalten, gefunden: '{line}'"));
    }
    let m = parts[0].parse::<usize>()
        .map_err(|_| format!("Ungültiges m: '{}'", parts[0]))?;
    let n = parts[1].parse::<usize>()
        .map_err(|_| format!("Ungültiges n: '{}'", parts[1]))?;
    Ok((m, n))
}

fn parse_constraint(line: &str, n: usize) -> Result<(Vec<f64>, f64), String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != n + 2 {
        return Err(format!(
            "Nebenbedingung hat {} Teile, erwartet {}: '{line}'",
            parts.len(), n + 2
        ));
    }
    if parts[n] != "<=" {
        return Err(format!("Erwartet '<=', gefunden '{}' in: '{line}'", parts[n]));
    }
    let coeffs = parse_coefficients(&parts[..n].join(" "), n)?;
    let rhs = parts[n + 1].parse::<f64>()
        .map_err(|_| format!("Ungültiger RHS-Wert: '{}' in: '{line}'", parts[n + 1]))?;
    Ok((coeffs, rhs))
}

fn parse_coefficients(line: &str, expected: usize) -> Result<Vec<f64>, String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != expected {
        return Err(format!(
            "Erwartet {expected} Koeffizienten, gefunden {}: '{line}'",
            parts.len()
        ));
    }
    parts.iter()
        .map(|s| s.parse::<f64>().map_err(|_| format!("Ungültige Zahl: '{s}'")))
        .collect()
}

fn main() {
    let path = Path::new("input.txt");
    match LinearProgram::from_file(path) {
        Ok(lp) => {
            let mut dict = SimplexDict::from_program(&lp);
            match dict.solve() {
                Ok(z) => dict.print_result(),
                Err(e) => eprintln!("Fehler: {e}"),
            }
        }
        Err(e) => eprintln!("Fehler: {e}"),
    }
}
