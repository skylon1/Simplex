use std::ops::Index;
use std::{fs, result};
use std::path::Path;

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
    pub x_n: Vec<f64>,
    pub a: Vec<Vec<f64>>,
    pub b: Vec<f64>,
    pub c: Vec<f64>,
    pub x_b: Vec<f64>,
    pub z_0: f64,
    pub z: f64,
    pub m: usize,
    pub n: usize,
}

impl SimplexDict {
    pub fn from_program(input: &LinearProgram) -> Self {
        let x_n = vec![0.0; input.n];
        let x_b = input.b.clone();
        let z_0 = 0.0;
        let z = 0.0;
        SimplexDict {
            x_n,
            a: input.a.clone(),
            b: input.b.clone(),
            c: input.c.clone(),
            x_b,
            z_0,
            z,
            m: input.m,
            n: input.n,
        }
    }
    pub fn solve(& mut self) -> Result<f64, string>{
        loop {
            
        }
    }
    pub fn step(& mut self){
        //pricing
        let max_id = self.c.iter().iter().enumerate().
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
        let first = lines
            .next()
            .ok_or("Datei ist leer")?;
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
        let obj_line = lines
            .next()
            .ok_or("Fehlende Zielfunktion")?;
        let c = parse_coefficients(obj_line, n)?;

        Ok(LinearProgram { a, b, c, m, n })
    }
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
    // Format: a1 a2 ... an <= bi
    let parts: Vec<&str> = line.split_whitespace().collect();

    // Erwarte n Koeffizienten + "<=" + 1 RHS = n+2 Teile
    if parts.len() != n + 2 {
        return Err(format!(
            "Nebenbedingung hat {} Teile, erwartet {} ({}  Koeffizienten + '<=' + RHS): '{line}'",
            parts.len(), n + 2, n
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
        Ok(lp) => println!("{lp:#?}"),
        Err(e) => eprintln!("Fehler: {e}"),
    }
}
