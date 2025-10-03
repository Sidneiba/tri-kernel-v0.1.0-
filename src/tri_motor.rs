// src/tri_motor.rs
// ====================
// MOTOR COGNITIVO TRI - LIA Inicial (3D Coordinates + Dispatcher)
// ====================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates(pub u8, pub u8, pub u8);  // x: estrutura, y: complexidade, z: padrão temporal

pub struct TriMotor;

impl TriMotor {
    /// Classificação heurística inicial baseada nos dados de entrada
    /// Futuro: Substituir por análise semântica real do LIA
    pub fn classify(data: &[u8]) -> Coordinates {
        let len = data.len();
        
        // Eixo X: Tipo de estrutura (0-4)
        let x = (len % 5) as u8;
        
        // Eixo Y: Complexidade estimada (0-2)
        let y = if len < 10 { 0 } 
               else if len < 50 { 1 } 
               else { 2 };
        
        // Eixo Z: Padrão temporal estimado (0-3)
        let unique_ratio = Self::estimate_uniqueness(data);
        let z = if unique_ratio > 0.8 { 3 }      // Alto padrão repetitivo
               else if unique_ratio > 0.5 { 2 }  // Padrão moderado  
               else { 1 };                       // Baixa repetição
        
        Coordinates(x, y, z)
    }
    
    /// Estima unicidade dos dados para padrão temporal
    fn estimate_uniqueness(data: &[u8]) -> f32 {
        if data.is_empty() { return 0.0; }
        
        let mut unique_count = 0;
        let _window_size = data.len().min(10);  // Fix: _ pra unused
        
        for i in 0..data.len().saturating_sub(1) {
            if data[i] != data[i + 1] {
                unique_count += 1;
            }
        }
        
        unique_count as f32 / data.len().max(1) as f32
    }

    /// Dispatcher universal - executa lógica baseada nas coordenadas 3D
    pub fn execute(coords: Coordinates, data: &[u8]) -> &[u8] {
        match (coords.0, coords.1, coords.2) {
            (1, 0, 3) | (1, 1, 3) => {
                // Perfil IoT: compressão agressiva
                let mut padded_data = [0u8; 32];
                let len = data.len().min(32);
                padded_data[..len].copy_from_slice(&data[..len]);
                let compressed = crate::tri_compress::compress(&padded_data);
                let comp_end = compressed.iter().position(|&x| x == 0).unwrap_or(64);
                // Fix: Retorna slice de static buffer (unsafe, mas kernel safe)
                unsafe { core::slice::from_raw_parts(compressed.as_ptr(), comp_end) }
            }
            (2, 2, 2) | (3, 2, 2) => {
                // Perfil IA: mantém estrutura
                data
            }
            (3, 1, 2) | (2, 1, 2) => {
                // Perfil streaming: transformação mínima
                data
            }
            _ => {
                // Default: pass-through
                data
            }
        }
    }

    /// Calcula ressonância entre estados temporais (simplificado)
    pub fn resonance_state(last: Option<Coordinates>, current: Coordinates) -> u8 {
        last.map_or(0, |last_coords| {
            let dx = (last_coords.0 as i16 - current.0 as i16).abs() as u8;
            let dy = (last_coords.1 as i16 - current.1 as i16).abs() as u8;
            let dz = (last_coords.2 as i16 - current.2 as i16).abs() as u8;
            (dx + dy + dz) / 3
        })
    }
}

// Interface pública
pub fn classify(data: &[u8]) -> Coordinates {
    TriMotor::classify(data)
}

pub fn execute(coords: Coordinates, data: &[u8]) -> &[u8] {
    TriMotor::execute(coords, data)
}

// Ciclo cognitivo completo (fix: full_cycle 1 arg, default None)
pub fn full_cycle(data: &[u8]) -> (Coordinates, &[u8], u8) {
    let coords = classify(data);
    let result = execute(coords, data);
    let resonance = TriMotor::resonance_state(None, coords);
    (coords, result, resonance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_iot_pattern() {
        let data = b"AAAAAABBBBBB";  // Len=12, repeats high, unique low ~0.25
        let coords = classify(data);
        assert_eq!(coords, Coordinates(2, 1, 1));  // x=12%5=2, y=1 (<50), z=1 (unique <0.5)
    }

    #[test]
    fn test_execute_compress() {
        let data = b"TESTDATA"; 
        let coords = Coordinates(1, 0, 3);  // Coordenada IoT
        let result = execute(coords, data);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_resonance_calculation() {
        let last = Some(Coordinates(1, 0, 3));
        let current = Coordinates(2, 1, 3);
        let resonance = TriMotor::resonance_state(last, current);
        assert_eq!(resonance, 0);  // dx=1, dy=1, dz=0 → (2)/3=0
    }
}
