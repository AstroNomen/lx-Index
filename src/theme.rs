use serde::{Deserialize, Serialize};
use slint::Color;
use crate::ThemeColors;

pub fn parse_hex_color(hex_str: &str) -> Color {
    let clean = hex_str.trim().trim_start_matches('#');
    if clean.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
        ) {
            return Color::from_argb_u8(255, r, g, b);
        }
    } else if clean.len() == 8 {
        if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
            u8::from_str_radix(&clean[6..8], 16),
        ) {
            return Color::from_argb_u8(a, r, g, b);
        }
    }
    Color::from_argb_u8(255, 76, 79, 105)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppTheme {
    pub id: String,
    pub name_en: String,
    pub name_zh: String,
    pub rosewater: String,
    pub flamingo: String,
    pub pink: String,
    pub mauve: String,
    pub red: String,
    pub maroon: String,
    pub peach: String,
    pub yellow: String,
    pub green: String,
    pub teal: String,
    pub sky: String,
    pub sapphire: String,
    pub blue: String,
    pub lavender: String,
    pub text: String,
    pub subtext1: String,
    pub subtext0: String,
    pub overlay2: String,
    pub overlay1: String,
    pub overlay0: String,
    pub surface2: String,
    pub surface1: String,
    pub surface0: String,
    pub base: String,
    pub mantle: String,
    pub crust: String,
    pub accent: String,
    pub accent_secondary: String,
    pub discard_bg: String,
    pub discard_fg: String,
    pub reset_bg: String,
    pub reset_hover_bg: String,
    pub reset_border: String,
    pub reset_fg: String,
}

impl AppTheme {
    /// 米黄古雅·羊皮纸书香 (Parchment - Default)
    pub fn parchment() -> Self {
        Self {
            id: "parchment".to_string(),
            name_en: "Parchment Warm".to_string(),
            name_zh: "米黄古雅 · 羊皮纸".to_string(),
            rosewater: "#b8685e".to_string(),
            flamingo: "#9e4e42".to_string(),
            pink: "#8f4258".to_string(),
            mauve: "#7a434d".to_string(), // 古典绛红/漆器红 (Antique Wine / Lacquer)
            red: "#a83232".to_string(),
            maroon: "#943838".to_string(),
            peach: "#b55a30".to_string(),  // 陶土赭石 (Terracotta)
            yellow: "#b87b1e".to_string(), // 典雅黄铜 (Antique Brass Amber)
            green: "#3e6b36".to_string(),  // 苔藓古绿 (Moss Forest)
            teal: "#2d6d66".to_string(),   // 汉玉青绿 (Antique Celadon)
            sky: "#387d99".to_string(),
            sapphire: "#286b7b".to_string(),
            blue: "#2f5d8a".to_string(),   // 普鲁士学术蓝 (Prussian Blue)
            lavender: "#65527a".to_string(),
            text: "#2c2523".to_string(),   // 深古墨色 (Rich Antique Ink, extremely legible)
            subtext1: "#453c37".to_string(),
            subtext0: "#5a4e47".to_string(),
            overlay2: "#756760".to_string(),
            overlay1: "#8f8078".to_string(),
            overlay0: "#a89a92".to_string(),
            surface2: "#c3b299".to_string(),
            surface1: "#d5c6b0".to_string(),
            surface0: "#e2d5c3".to_string(),
            base: "#fbf5ea".to_string(),   // 温暖米黄底 (Warm Light Parchment)
            mantle: "#f4ece1".to_string(), // 次级羊皮纸色
            crust: "#ebe1d3".to_string(),  // 顶栏装订色
            accent: "#7a434d".to_string(), // 古典绛红/漆器酒红 (Antique Lacquer Red)
            accent_secondary: "#b55a30".to_string(), // 陶土赭石 (Terracotta)
            discard_bg: "#6b585022".to_string(),
            discard_fg: "#3e2e28".to_string(), // 纯正深褐色废弃色，高对比高可读
            reset_bg: "#e2d5c366".to_string(),
            reset_hover_bg: "#e2d5c3".to_string(),
            reset_border: "#c3b299".to_string(),
            reset_fg: "#453c37".to_string(),   // 雅致书卷次级色，非红色
        }
    }

    /// 经典学术·Solarized Light
    pub fn solarized_light() -> Self {
        Self {
            id: "solarized_light".to_string(),
            name_en: "Solarized Light".to_string(),
            name_zh: "经典学术 · Solarized Light".to_string(),
            rosewater: "#b58900".to_string(),
            flamingo: "#cb4b16".to_string(),
            pink: "#d33682".to_string(),
            mauve: "#6c71c4".to_string(),
            red: "#dc322f".to_string(),
            maroon: "#dc322f".to_string(),
            peach: "#cb4b16".to_string(),
            yellow: "#b58900".to_string(),
            green: "#859900".to_string(),
            teal: "#2aa198".to_string(),
            sky: "#2aa198".to_string(),
            sapphire: "#268bd2".to_string(),
            blue: "#268bd2".to_string(),
            lavender: "#6c71c4".to_string(),
            text: "#073642".to_string(),   // base03
            subtext1: "#586e75".to_string(), // base01
            subtext0: "#657b83".to_string(), // base00
            overlay2: "#839496".to_string(),
            overlay1: "#93a1a1".to_string(),
            overlay0: "#a4b2b2".to_string(),
            surface2: "#b5aa8e".to_string(),
            surface1: "#c4baa2".to_string(),
            surface0: "#d3cbb7".to_string(),
            base: "#fdf6e3".to_string(),   // base3
            mantle: "#eee8d5".to_string(), // base2
            crust: "#e0d8c3".to_string(),
            accent: "#268bd2".to_string(), // Academic Blue
            accent_secondary: "#2aa198".to_string(), // Celadon Teal
            discard_bg: "#586e7525".to_string(),
            discard_fg: "#073642".to_string(),
            reset_bg: "#eee8d5".to_string(),
            reset_hover_bg: "#d3cbb7".to_string(),
            reset_border: "#b5aa8e".to_string(),
            reset_fg: "#586e75".to_string(),
        }
    }

    /// 线装古籍·象牙宣纸 (Antique Ivory)
    pub fn antique_ivory() -> Self {
        Self {
            id: "antique_ivory".to_string(),
            name_en: "Antique Ivory".to_string(),
            name_zh: "线装古籍 · 象牙宣纸".to_string(),
            rosewater: "#d65d0e".to_string(),
            flamingo: "#af3a03".to_string(),
            pink: "#b16286".to_string(),
            mauve: "#8f3f71".to_string(),
            red: "#9d0006".to_string(),
            maroon: "#9d0006".to_string(),
            peach: "#af3a03".to_string(),
            yellow: "#b57614".to_string(),
            green: "#79740e".to_string(),
            teal: "#427b58".to_string(),
            sky: "#458588".to_string(),
            sapphire: "#076678".to_string(),
            blue: "#076678".to_string(),
            lavender: "#458588".to_string(),
            text: "#282828".to_string(),   // 徽墨黑 (Deep soot ink)
            subtext1: "#3c3836".to_string(),
            subtext0: "#504945".to_string(),
            overlay2: "#665c54".to_string(),
            overlay1: "#7c6f64".to_string(),
            overlay0: "#928374".to_string(),
            surface2: "#a89984".to_string(),
            surface1: "#bdae93".to_string(),
            surface0: "#d5c4a1".to_string(),
            base: "#fbf1c7".to_string(),   // 宣纸象牙黄 (Warm Xuan paper)
            mantle: "#f2e5bc".to_string(),
            crust: "#ebdbb2".to_string(),
            accent: "#9d0006".to_string(), // 徽墨朱砂红 (Cinnabar Rust Red)
            accent_secondary: "#b57614".to_string(), // 典雅黄铜 (Antique Brass Amber)
            discard_bg: "#665c5425".to_string(),
            discard_fg: "#282828".to_string(),
            reset_bg: "#ebdbb2".to_string(),
            reset_hover_bg: "#d5c4a1".to_string(),
            reset_border: "#a89984".to_string(),
            reset_fg: "#3c3836".to_string(),
        }
    }

    /// 现代清爽·Catppuccin Latte
    pub fn catppuccin_latte() -> Self {
        Self {
            id: "catppuccin_latte".to_string(),
            name_en: "Catppuccin Latte".to_string(),
            name_zh: "现代清爽 · Catppuccin Latte".to_string(),
            rosewater: "#dc8a78".to_string(),
            flamingo: "#dd7878".to_string(),
            pink: "#ea76cb".to_string(),
            mauve: "#8839ef".to_string(),
            red: "#d20f39".to_string(),
            maroon: "#e64553".to_string(),
            peach: "#fe640b".to_string(),
            yellow: "#df8e1d".to_string(),
            green: "#40a02b".to_string(),
            teal: "#179299".to_string(),
            sky: "#04a5e5".to_string(),
            sapphire: "#209fb5".to_string(),
            blue: "#1e66f5".to_string(),
            lavender: "#7287fd".to_string(),
            text: "#4c4f69".to_string(),
            subtext1: "#5c5f77".to_string(),
            subtext0: "#6c6f85".to_string(),
            overlay2: "#7c7f93".to_string(),
            overlay1: "#8c8fa1".to_string(),
            overlay0: "#9ca0b0".to_string(),
            surface2: "#acb0be".to_string(),
            surface1: "#bcc0cc".to_string(),
            surface0: "#ccd0da".to_string(),
            base: "#eff1f5".to_string(),
            mantle: "#e6e9ef".to_string(),
            crust: "#dce0e8".to_string(),
            accent: "#8839ef".to_string(), // Mauve Purple
            accent_secondary: "#ea76cb".to_string(), // Pink
            discard_bg: "#5c5f7725".to_string(),
            discard_fg: "#2a2d40".to_string(),
            reset_bg: "#ccd0da55".to_string(),
            reset_hover_bg: "#bcc0cc".to_string(),
            reset_border: "#acb0be".to_string(),
            reset_fg: "#5c5f77".to_string(),
        }
    }

    /// 典雅玫瑰·Rosé Pine Dawn
    pub fn rose_pine_dawn() -> Self {
        Self {
            id: "rose_pine_dawn".to_string(),
            name_en: "Rosé Pine Dawn".to_string(),
            name_zh: "典雅暮光 · Rosé Pine Dawn".to_string(),
            rosewater: "#ea9d34".to_string(),
            flamingo: "#d7827e".to_string(),
            pink: "#eb6f92".to_string(),
            mauve: "#907aa9".to_string(),  // iris
            red: "#b4637a".to_string(),    // love
            maroon: "#b4637a".to_string(),
            peach: "#d7827e".to_string(),  // rose
            yellow: "#ea9d34".to_string(), // gold
            green: "#286983".to_string(),  // pine
            teal: "#56949f".to_string(),   // foam
            sky: "#56949f".to_string(),
            sapphire: "#286983".to_string(),
            blue: "#286983".to_string(),
            lavender: "#907aa9".to_string(),
            text: "#575279".to_string(),   // dusky text
            subtext1: "#797593".to_string(), // subtle
            subtext0: "#9893a5".to_string(), // muted
            overlay2: "#b5b0c0".to_string(),
            overlay1: "#c5c0d0".to_string(),
            overlay0: "#d5d0e0".to_string(),
            surface2: "#cecacd".to_string(),
            surface1: "#dfdad9".to_string(),
            surface0: "#e4dfde".to_string(),
            base: "#faf4ed".to_string(),   // base
            mantle: "#fffaf3".to_string(), // surface
            crust: "#f2e9de".to_string(),  // overlay
            accent: "#907aa9".to_string(), // Iris Purple
            accent_secondary: "#eb6f92".to_string(), // Love Rose
            discard_bg: "#79759325".to_string(),
            discard_fg: "#3d385a".to_string(),
            reset_bg: "#f2e9de".to_string(),
            reset_hover_bg: "#dfdad9".to_string(),
            reset_border: "#cecacd".to_string(),
            reset_fg: "#575279".to_string(),
        }
    }

    pub fn by_id(id: &str) -> Self {
        match id {
            "solarized_light" | "solarized" => Self::solarized_light(),
            "antique_ivory" | "ivory" => Self::antique_ivory(),
            "rose_pine_dawn" | "rose-pine" | "rose_pine" => Self::rose_pine_dawn(),
            "catppuccin_latte" | "latte" | "catppuccin" => Self::catppuccin_latte(),
            _ => Self::parchment(),
        }
    }

    pub fn all_themes() -> Vec<Self> {
        vec![
            Self::parchment(),
            Self::rose_pine_dawn(),
            Self::solarized_light(),
            Self::antique_ivory(),
            Self::catppuccin_latte(),
        ]
    }

    /// Converts into Slint ThemeColors struct.
    pub fn to_slint_theme(&self) -> ThemeColors {
        ThemeColors {
            rosewater: parse_hex_color(&self.rosewater),
            flamingo: parse_hex_color(&self.flamingo),
            pink: parse_hex_color(&self.pink),
            mauve: parse_hex_color(&self.mauve),
            red: parse_hex_color(&self.red),
            maroon: parse_hex_color(&self.maroon),
            peach: parse_hex_color(&self.peach),
            yellow: parse_hex_color(&self.yellow),
            green: parse_hex_color(&self.green),
            teal: parse_hex_color(&self.teal),
            sky: parse_hex_color(&self.sky),
            sapphire: parse_hex_color(&self.sapphire),
            blue: parse_hex_color(&self.blue),
            lavender: parse_hex_color(&self.lavender),
            text: parse_hex_color(&self.text),
            subtext1: parse_hex_color(&self.subtext1),
            subtext0: parse_hex_color(&self.subtext0),
            overlay2: parse_hex_color(&self.overlay2),
            overlay1: parse_hex_color(&self.overlay1),
            overlay0: parse_hex_color(&self.overlay0),
            surface2: parse_hex_color(&self.surface2),
            surface1: parse_hex_color(&self.surface1),
            surface0: parse_hex_color(&self.surface0),
            base: parse_hex_color(&self.base),
            mantle: parse_hex_color(&self.mantle),
            crust: parse_hex_color(&self.crust),
            accent: parse_hex_color(&self.accent),
            accent_secondary: parse_hex_color(&self.accent_secondary),
            discard_bg: parse_hex_color(&self.discard_bg),
            discard_fg: parse_hex_color(&self.discard_fg),
            reset_bg: parse_hex_color(&self.reset_bg),
            reset_hover_bg: parse_hex_color(&self.reset_hover_bg),
            reset_border: parse_hex_color(&self.reset_border),
            reset_fg: parse_hex_color(&self.reset_fg),
        }
    }
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::parchment()
    }
}
