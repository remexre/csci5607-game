use failure::{Error, Fallible};
use std::{path::PathBuf, str::FromStr};

/// The map as parsed.
#[derive(Debug, Deserialize, Serialize)]
pub struct Map {
    /// The dimensions of the map.
    pub dims: (usize, usize),

    /// The actual floor layout.
    pub floor: Vec<Tile>,

    /// The location of keys.
    pub keys: Vec<(usize, usize, char)>,

    /// The filename of the material used for the floor.
    pub material_floor: Option<PathBuf>,

    /// The filename of the material used for walls.
    pub material_wall: Option<PathBuf>,

    /// The filename of the character model.
    pub model_character: Option<PathBuf>,

    /// The filename of the key model.
    pub model_key: PathBuf,
}

impl FromStr for Map {
    type Err = Error;

    fn from_str(s: &str) -> Fallible<Map> {
        let w_end_idx = s.find(' ').unwrap();
        let h_end_idx = s[w_end_idx..].find('\n').unwrap() + w_end_idx;

        let w = &s[..w_end_idx];
        let h = &s[w_end_idx + 1..h_end_idx];
        let mut map = Map {
            dims: (w.parse()?, h.parse()?),
            floor: Vec::new(),
            keys: Vec::new(),

            material_floor: None,
            material_wall: None,
            model_character: None,
            model_key: PathBuf::from("key.obj"),
        };

        let mut rest = &s[h_end_idx + 1..];
        let mut x = 0;
        let mut y = 0;
        while map.floor.len() != map.dims.0 * map.dims.1 {
            let ch = rest
                .chars()
                .next()
                .ok_or_else(|| format_err!("Unexpected EOF while parsing map body"))?;
            parse_tile(&mut map, ch, x, y)?;
            x += 1;
            if x > map.dims.0 {
                x = 0;
                y += 1;
            }
            rest = &rest[1..];
        }

        rest = rest.trim_left();
        if rest.is_empty() {
            Ok(map)
        } else {
            bail!("Expected EOF, found {:?}", rest)
        }
    }
}

/// The floor map tile.
#[derive(Debug, Deserialize, Serialize)]
pub enum Tile {
    /// An empty floor tile.
    #[serde(rename = "e")]
    Empty,

    /// A wall.
    #[serde(rename = "w")]
    Wall,

    /// A door with the given character assigned to it.
    #[serde(rename = "d")]
    Door(char),
}

fn parse_tile(map: &mut Map, ch: char, x: usize, y: usize) -> Fallible<()> {
    let tile = match ch {
        '0' | 'G' | 'S' => Tile::Empty,
        'A'...'E' => Tile::Door(ch),
        'a'...'e' => {
            map.keys.push((x, y, ch));
            Tile::Empty
        }
        'W' => Tile::Wall,
        '\n' | '\r' | '\t' | ' ' => return Ok(()),
        _ => bail!("Invalid tile {:?}", ch),
    };
    map.floor.push(tile);
    Ok(())
}
