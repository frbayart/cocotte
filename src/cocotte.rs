use handlebars::Handlebars;
use palette::{Hsl, RgbHue};
use palette::rgb::Rgb;
use palette::FromColor;
use std::iter;

pub fn render(template: &str, source: &str, hue: Hsl, verbose: u16) {
    let (red, green, blue) = hsl_to_rgb(hue);
    let mut reg = Handlebars::new();
    handlebars_helper!(hex2: |v: i64| format!("{:02X}", v));
    reg.register_helper("hex2", Box::new(hex2));

    if verbose > 0 {
        println!("{:?}", hue);
    }

    match reg.render_template(
        template,
        &json!({"red": red, "green": green, "blue": blue, "source": source}),
    ) {
        Ok(out) => print!("{}", out),
        Err(sth) => println!("Error: {}", sth),
    }
}

pub static FORMAT_ITERM_BG: &str = "iterm_bg";
pub static FORMAT_ITERM_TAB: &str = "iterm_tab";
pub static FORMAT_HTML: &str = "html";

pub fn get_format(format: Option<&str>) -> &str {
    match format {
        Some(t) => {
            match t {
                "iterm_bg" => {
                    "\x1b]1337;SetColors=bg={{hex2 red}}{{ hex2 green}}{{ hex2 blue}}\x07"
                }
                "iterm_tab" => {
                    "\x1b]6;1;bg;red;brightness;{{ red }}\x07\x1b]6;1;bg;green;brightness;{{ green }}\x07\x1b]6;1;bg;blue;brightness;{{ blue }}\x07"
                }
                _ => {
                    "<div style='background-color: 0x{{hex2 red}}{{hex2 green}}{{hex2 blue}};'>{{red}} {{green}} {{blue}} {{source}}</div>\n"
                }
            }
        }
        None => {
            "<div style='background-color: 0x{{hex2 red}}{{hex2 green}}{{hex2 blue}};'>{{red}} {{green}} {{blue}} {{source}}</div>\n"
        }
    }
}

fn hsl_to_rgb(hue: Hsl) -> (u8, u8, u8) {
    let rgbc: Rgb = Rgb::from_hsl(hue);
    // this should be >= 0.0
    let r = (rgbc.red.max(0.0) * 255.0) as u8;
    let g = (rgbc.green.max(0.0) * 255.0) as u8;
    let b = (rgbc.blue.max(0.0) * 255.0) as u8;
    (r, g, b)
}

pub fn hsl(path: &str, black_list: &Vec<String>, verbose: u16) -> Hsl {

    let cleaned_path = clean(path, black_list);

    if verbose > 0 {
        println!("Path components after filtering: {:?}", cleaned_path);
    }

    Hsl::new(RgbHue::from(hue_for(cleaned_path.concat())), 100.0, 0.5)
}

pub fn clean<'a>(source: &'a str, black_list: &Vec<String>) -> Vec<&'a str> {
    let str_black_list: Vec<&str> = black_list.iter().map(|i| i.as_str()).collect();

    source
        .split('/')
        .filter(|it| it.len() > 0 && !str_black_list.contains(it))
        .collect()
}

pub fn hue_for(source: String) -> f32 {

    let ascii_source = source.to_ascii_lowercase();
    let mut hue = 0.0;
    let p = positioner();

    for (ix, c) in ascii_source.as_bytes().into_iter().enumerate() {
        let uc = *c as usize;
        let (pos, tot) = p.position(uc);
        let factor = match ix {
            0...9 => 36.0,
            _ => 0.4,
        };

        hue = hue + factor * (pos as f32) / (tot as f32);
    }
    hue
}

struct Positioner {
    index: Vec<usize>,
}

fn positioner() -> Positioner {
    let allowed_ranges: Vec<Vec<usize>> = vec![
        vec![48, 58], // 0 to :
        vec![97, 123], // a-z
    ];

    let indexer: Vec<usize> = allowed_ranges
        .into_iter()
        .map(|range| {
            iter::repeat(0)
                .take(range[1] - range[0])
                .enumerate()
                .map(|(ix, _b)| range[0] + ix)
                .collect()
        })
        .flat_map(|s: Vec<usize>| s)
        .collect();

    Positioner { index: indexer }
}

impl Positioner {
    // Return the char position
    fn position(&self, chr: usize) -> (usize, usize) {
        (
            self.index.iter().position(|&x| x == chr).unwrap_or(0),
            self.index.len(),
        )
    }
}

#[test]
fn test() {
    let p = positioner();
    assert_eq!(p.position('0' as usize), (0, 36));
    assert_eq!(p.position('1' as usize), (1, 36));
    assert_eq!(p.position('z' as usize), (35, 36));

}
