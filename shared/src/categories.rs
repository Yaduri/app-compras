use serde::{Deserialize, Serialize};

/// Identificador único e canônico das categorias de supermercado
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CategoryId {
    Produce,
    Butcher,
    Dairy,
    Bakery,
    Grocery,
    Beverages,
    Cleaning,
    Hygiene,
    Frozen,
    Other,
}

impl CategoryId {
    pub fn as_str(&self) -> &'static str {
        match self {
            CategoryId::Produce => "produce",
            CategoryId::Butcher => "butcher",
            CategoryId::Dairy => "dairy",
            CategoryId::Bakery => "bakery",
            CategoryId::Grocery => "grocery",
            CategoryId::Beverages => "beverages",
            CategoryId::Cleaning => "cleaning",
            CategoryId::Hygiene => "hygiene",
            CategoryId::Frozen => "frozen",
            CategoryId::Other => "other",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CategoryId::Produce => "Hortifruti",
            CategoryId::Butcher => "Açougue e Carnes",
            CategoryId::Dairy => "Laticínios e Frios",
            CategoryId::Bakery => "Padaria e Confeitaria",
            CategoryId::Grocery => "Mercearia e Grãos",
            CategoryId::Beverages => "Bebidas",
            CategoryId::Cleaning => "Limpeza",
            CategoryId::Hygiene => "Higiene Pessoal",
            CategoryId::Frozen => "Congelados",
            CategoryId::Other => "Outros e Diversos",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            CategoryId::Produce => "apple",
            CategoryId::Butcher => "drumstick-bite",
            CategoryId::Dairy => "cheese",
            CategoryId::Bakery => "croissant",
            CategoryId::Grocery => "wheat-awn",
            CategoryId::Beverages => "wine-bottle",
            CategoryId::Cleaning => "sparkles",
            CategoryId::Hygiene => "pump-soap",
            CategoryId::Frozen => "snowflake",
            CategoryId::Other => "tags",
        }
    }

    pub fn color_hex(&self) -> &'static str {
        match self {
            CategoryId::Produce => "#16a34a",   // Verde
            CategoryId::Butcher => "#dc2626",   // Vermelho
            CategoryId::Dairy => "#eab308",     // Amarelo
            CategoryId::Bakery => "#d97706",    // Âmbar
            CategoryId::Grocery => "#854d0e",   // Marrom Claro
            CategoryId::Beverages => "#0284c7", // Azul Claro
            CategoryId::Cleaning => "#0891b2",  // Ciano
            CategoryId::Hygiene => "#7c3aed",   // Roxo
            CategoryId::Frozen => "#2563eb",    // Azul Forte
            CategoryId::Other => "#64748b",     // Cinza Ardósia
        }
    }

    pub fn sort_order(&self) -> i32 {
        match self {
            CategoryId::Produce => 1,
            CategoryId::Butcher => 2,
            CategoryId::Dairy => 3,
            CategoryId::Bakery => 4,
            CategoryId::Grocery => 5,
            CategoryId::Beverages => 6,
            CategoryId::Cleaning => 7,
            CategoryId::Hygiene => 8,
            CategoryId::Frozen => 9,
            CategoryId::Other => 10,
        }
    }

    pub fn from_id_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "produce" => CategoryId::Produce,
            "butcher" => CategoryId::Butcher,
            "dairy" => CategoryId::Dairy,
            "bakery" => CategoryId::Bakery,
            "grocery" => CategoryId::Grocery,
            "beverages" => CategoryId::Beverages,
            "cleaning" => CategoryId::Cleaning,
            "hygiene" => CategoryId::Hygiene,
            "frozen" => CategoryId::Frozen,
            _ => CategoryId::Other,
        }
    }

    /// Retorna a lista completa com todas as categorias disponíveis ordenadas
    pub fn all() -> &'static [CategoryId] {
        &[
            CategoryId::Produce,
            CategoryId::Butcher,
            CategoryId::Dairy,
            CategoryId::Bakery,
            CategoryId::Grocery,
            CategoryId::Beverages,
            CategoryId::Cleaning,
            CategoryId::Hygiene,
            CategoryId::Frozen,
            CategoryId::Other,
        ]
    }

    /// Algoritmo inteligente de autocategorização em português
    /// Normaliza a string (remoção de acentos e pontuações) e busca por palavras-chave
    pub fn detect_from_name(name: &str) -> CategoryId {
        let normalized = normalize_text(name);
        if normalized.is_empty() {
            return CategoryId::Other;
        }

        // Prioridade 1: Produtos com nomes compostos ou termos de limpeza/higiene
        // (ex: "detergente de maca" deve ser Limpeza e não Produce)
        const CLEANING_KEYWORDS: &[&str] = &[
            "detergente", "sabao", "amaciante", "desinfetante", "agua sanitaria", "alvejante",
            "esponja", "bucha", "palha de aco", "bombril", "pano de prato", "pano de chao",
            "saco de lixo", "lixeira", "limpa vidro", "lustra moveis", "multiuso", "cloro",
            "desengordurante", "inseticida", "aromatizador", "odorizador", "vassoura", "rodo",
            "limpador", "sanitario", "candida", "vanish", "ype", "ajax", "veja", "omoo", "omo",
            "brilhante", "comfort", "downy",
        ];

        for &kw in CLEANING_KEYWORDS {
            if contains_word_or_phrase(&normalized, kw) {
                return CategoryId::Cleaning;
            }
        }

        const HYGIENE_KEYWORDS: &[&str] = &[
            "sabonete", "shampoo", "xampu", "condicionador", "desodorante", "pasta de dente",
            "creme dental", "escova de dente", "fio dental", "papel higienico", "absorvente",
            "lamina de barbear", "barbeador", "creme de barbear", "hidratante", "protetor solar",
            "cotonete", "algodao", "enxaguante bucal", "listerine", "colgate", "pantene", "dove",
            "rexona", "nivea", "gillette", "fralda", "lenco umedecido", "antisseptico",
        ];

        for &kw in HYGIENE_KEYWORDS {
            if contains_word_or_phrase(&normalized, kw) {
                return CategoryId::Hygiene;
            }
        }

        // Prioridade 2: Congelados
        const FROZEN_KEYWORDS: &[&str] = &[
            "sorvete", "picole", "acai", "pizza congelada", "lasanha congelada", "hamburguer",
            "nuggets", "empanado", "batata frita congelada", "polpa de fruta", "congelad",
            "gelo", "waffle congelado",
        ];

        for &kw in FROZEN_KEYWORDS {
            if contains_word_or_phrase(&normalized, kw) {
                return CategoryId::Frozen;
            }
        }

        // Prioridade 3: Açougue e Carnes
        const BUTCHER_KEYWORDS: &[&str] = &[
            "picanha", "alcatra", "carne", "maminha", "contrafile", "file mignon", "acem",
            "patinho", "costela", "coxao mole", "coxao duro", "lagarto", "fraldinha",
            "frango", "peito de frango", "coxa de frango", "sobrecoxa", "asa de frango", "file de frango",
            "linguica", "bisteca", "lombo", "pernil", "suino", "porco",
            "peixe", "salmao", "tilapia", "bacalhau", "camarao", "atum", "sardinha", "merluza",
            "bacon", "presunto", "mortadela", "salame", "carne moida", "costelinha", "tender",
            "peru", "chester", "calabresa",
        ];

        for &kw in BUTCHER_KEYWORDS {
            if contains_word_or_phrase(&normalized, kw) {
                return CategoryId::Butcher;
            }
        }

        // Prioridade 4: Laticínios e Frios
        const DAIRY_KEYWORDS: &[&str] = &[
            "leite", "queijo", "mussarela", "mucarela", "prato", "parmesao", "gorgonzola",
            "provolone", "brie", "requeijao", "iogurte", "manteiga", "margarina", "creme de leite",
            "leite condensado", "ricota", "coalhada", "nata", "catupiry", "danone", "yakult",
            "cream cheese", "cheddar", "leite em po",
        ];

        for &kw in DAIRY_KEYWORDS {
            if contains_word_or_phrase(&normalized, kw) {
                return CategoryId::Dairy;
            }
        }

        // Prioridade 5: Padaria e Confeitaria
        const BAKERY_KEYWORDS: &[&str] = &[
            "pao", "pao frances", "pao de forma", "pao de queijo", "baguete", "croissant",
            "bolo", "torta", "biscoito", "bolacha", "torrada", "broa", "sonho", "rosquinha",
            "brioche", "pastel", "muffin", "cookie", "panetone", "chocotone",
        ];

        for &kw in BAKERY_KEYWORDS {
            if contains_word_or_phrase(&normalized, kw) {
                return CategoryId::Bakery;
            }
        }

        // Prioridade 6: Hortifruti (Frutas, Legumes e Verduras)
        const PRODUCE_KEYWORDS: &[&str] = &[
            "maca", "banana", "laranja", "limao", "tomate", "batata", "cebola", "alho",
            "cenoura", "alface", "brocolis", "abobora", "manga", "uva", "melancia", "melao",
            "pera", "morango", "abacaxi", "pimentao", "rucula", "espinafre", "pepino",
            "mandioca", "aipim", "couve", "couve-flor", "beterraba", "maracuja", "abacate",
            "kiwi", "tangerina", "mexerica", "bergamota", "berinjela", "chuchu", "coentro",
            "salsa", "salsinha", "cebolinha", "hortela", "gengibre", "repolho", "quiabo",
            "vagem", "maca verde", "pessego", "ameixa", "mamao", "goiaba", "caju", "caqui",
        ];

        for &kw in PRODUCE_KEYWORDS {
            if contains_word_or_phrase(&normalized, kw) {
                return CategoryId::Produce;
            }
        }

        // Prioridade 7: Bebidas
        const BEVERAGES_KEYWORDS: &[&str] = &[
            "agua", "refrigerante", "coca", "coca-cola", "pepsi", "guarana", "fanta", "sprite",
            "suco", "cha", "cerveja", "vinho", "vodka", "gin", "whisky", "energetico",
            "red bull", "monster", "espumante", "soda", "tonica", "h2oh", "gatorade", "isotonico",
            "nectar", "rum", "tequila",
        ];

        for &kw in BEVERAGES_KEYWORDS {
            if contains_word_or_phrase(&normalized, kw) {
                return CategoryId::Beverages;
            }
        }

        // Prioridade 8: Mercearia e Grãos
        const GROCERY_KEYWORDS: &[&str] = &[
            "arroz", "feijao", "macarrao", "oleo", "azeite", "acucar", "sal", "cafe",
            "farinha", "trigo", "milho", "molho de tomate", "extrato de tomate", "ervilha",
            "milho verde", "azeitona", "maionese", "ketchup", "mostarda", "vinagre",
            "fermento", "tempero", "caldo knorr", "aveia", "granola", "cereal", "lentilha",
            "grao de bico", "achocolatado", "toddy", "nescau", "gelatina", "doce de leite",
            "geleia", "palmito", "sardinha em lata", "atum em lata", "pipoca", "canela",
            "oregano", "pimenta", "shoyu",
        ];

        for &kw in GROCERY_KEYWORDS {
            if contains_word_or_phrase(&normalized, kw) {
                return CategoryId::Grocery;
            }
        }

        CategoryId::Other
    }
}

/// Normaliza o texto removendo acentos, diacríticos e pontuação para busca uniforme
pub fn normalize_text(input: &str) -> String {
    let lower = input.to_lowercase();
    let mut result = String::with_capacity(lower.len());

    for ch in lower.chars() {
        match ch {
            'á' | 'à' | 'ã' | 'â' | 'ä' => result.push('a'),
            'é' | 'è' | 'ê' | 'ë' => result.push('e'),
            'í' | 'ì' | 'î' | 'ï' => result.push('i'),
            'ó' | 'ò' | 'õ' | 'ô' | 'ö' => result.push('o'),
            'ú' | 'ù' | 'û' | 'ü' => result.push('u'),
            'ç' => result.push('c'),
            'ñ' => result.push('n'),
            c if c.is_alphanumeric() || c.is_whitespace() || c == '-' => result.push(c),
            _ => result.push(' '),
        }
    }

    result.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// Verifica se a palavra ou frase está contida respeitando limites de palavras ou plurais comuns
fn contains_word_or_phrase(haystack: &str, needle: &str) -> bool {
    let h_len = haystack.len();
    let n_len = needle.len();

    if n_len > h_len {
        return false;
    }

    // Se needle contém espaço (ex: "peito de frango", "agua sanitaria"), procuramos substring
    if needle.contains(' ') {
        return haystack.contains(needle);
    }

    for (idx, _) in haystack.match_indices(needle) {
        let before_ok = idx == 0 || haystack.as_bytes()[idx - 1] == b' ';
        if !before_ok {
            continue;
        }

        let after_idx = idx + n_len;
        // Correspondência exata da palavra
        if after_idx == h_len || haystack.as_bytes()[after_idx] == b' ' {
            return true;
        }

        // Plural com 's' (ex: "tomates maduros", "bananas")
        if haystack.as_bytes()[after_idx] == b's' {
            let next_idx = after_idx + 1;
            if next_idx == h_len || haystack.as_bytes()[next_idx] == b' ' {
                return true;
            }
        }

        // Plural com 'es' (ex: "limoes", "hamburgueres")
        if after_idx + 2 <= h_len && &haystack[after_idx..after_idx + 2] == "es" {
            let next_idx = after_idx + 2;
            if next_idx == h_len || haystack.as_bytes()[next_idx] == b' ' {
                return true;
            }
        }
    }

    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryInfo {
    pub id: CategoryId,
    pub name: String,
    pub icon: String,
    pub color: String,
    pub sort_order: i32,
}

impl From<CategoryId> for CategoryInfo {
    fn from(id: CategoryId) -> Self {
        Self {
            id,
            name: id.display_name().to_string(),
            icon: id.icon_name().to_string(),
            color: id.color_hex().to_string(),
            sort_order: id.sort_order(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autocategorization_produce() {
        assert_eq!(CategoryId::detect_from_name("Maçã Gala"), CategoryId::Produce);
        assert_eq!(CategoryId::detect_from_name("Banana prata 1 dúzia"), CategoryId::Produce);
        assert_eq!(CategoryId::detect_from_name("Cenoura fresca"), CategoryId::Produce);
        assert_eq!(CategoryId::detect_from_name("Tomates maduros"), CategoryId::Produce);
    }

    #[test]
    fn test_autocategorization_butcher() {
        assert_eq!(CategoryId::detect_from_name("Picanha Bovina"), CategoryId::Butcher);
        assert_eq!(CategoryId::detect_from_name("Peito de Frango Congelado"), CategoryId::Butcher);
        assert_eq!(CategoryId::detect_from_name("Linguiça Toscana"), CategoryId::Butcher);
        assert_eq!(CategoryId::detect_from_name("Salmão fresco"), CategoryId::Butcher);
    }

    #[test]
    fn test_autocategorization_dairy() {
        assert_eq!(CategoryId::detect_from_name("Leite Integral"), CategoryId::Dairy);
        assert_eq!(CategoryId::detect_from_name("Queijo Mussarela fatiado"), CategoryId::Dairy);
        assert_eq!(CategoryId::detect_from_name("Requeijão Cremoso"), CategoryId::Dairy);
        assert_eq!(CategoryId::detect_from_name("Manteiga com sal"), CategoryId::Dairy);
    }

    #[test]
    fn test_autocategorization_bakery() {
        assert_eq!(CategoryId::detect_from_name("Pão Francês"), CategoryId::Bakery);
        assert_eq!(CategoryId::detect_from_name("Pão de Forma Integral"), CategoryId::Bakery);
        assert_eq!(CategoryId::detect_from_name("Bolo de Chocolate"), CategoryId::Bakery);
    }

    #[test]
    fn test_autocategorization_cleaning_priority() {
        // "Detergente de maçã" não pode ir para Produce, deve ir para Cleaning!
        assert_eq!(CategoryId::detect_from_name("Detergente Ypê Maçã"), CategoryId::Cleaning);
        assert_eq!(CategoryId::detect_from_name("Sabão em pó Omo 1kg"), CategoryId::Cleaning);
        assert_eq!(CategoryId::detect_from_name("Água Sanitária 2L"), CategoryId::Cleaning);
    }

    #[test]
    fn test_autocategorization_hygiene() {
        assert_eq!(CategoryId::detect_from_name("Shampoo Anticaspa"), CategoryId::Hygiene);
        assert_eq!(CategoryId::detect_from_name("Pasta de dente Colgate"), CategoryId::Hygiene);
        assert_eq!(CategoryId::detect_from_name("Sabonete Dove"), CategoryId::Hygiene);
        assert_eq!(CategoryId::detect_from_name("Papel Higiênico 12 rolos"), CategoryId::Hygiene);
    }

    #[test]
    fn test_autocategorization_beverages() {
        assert_eq!(CategoryId::detect_from_name("Coca-Cola 2L"), CategoryId::Beverages);
        assert_eq!(CategoryId::detect_from_name("Cerveja Heineken"), CategoryId::Beverages);
        assert_eq!(CategoryId::detect_from_name("Água Mineral sem gás"), CategoryId::Beverages);
    }

    #[test]
    fn test_autocategorization_grocery() {
        assert_eq!(CategoryId::detect_from_name("Arroz Branco Tipo 1"), CategoryId::Grocery);
        assert_eq!(CategoryId::detect_from_name("Feijão Carioca 1kg"), CategoryId::Grocery);
        assert_eq!(CategoryId::detect_from_name("Azeite de Oliva Extra Virgem"), CategoryId::Grocery);
        assert_eq!(CategoryId::detect_from_name("Café Torrado e Moído"), CategoryId::Grocery);
    }

    #[test]
    fn test_autocategorization_unknown_fallback() {
        assert_eq!(CategoryId::detect_from_name("Giz de cera escolar"), CategoryId::Other);
        assert_eq!(CategoryId::detect_from_name(""), CategoryId::Other);
    }
}
