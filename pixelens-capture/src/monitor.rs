fn parse_xy(s: &str) -> PixelensResult<(i32, i32)> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(PixelensError::Config(format!("invalid cursorpos format: {}", s)));
    }
    let x = parts[0].trim().parse().map_err(|e| PixelensError::Config(format!("parse x: {e}")))?;
    let y = parts[1].trim().parse().map_err(|e| PixelensError::Config(format!("parse y: {e}")))?;
    Ok((x, y))
}

fn parse_xdotool_pos(s: &str) -> PixelensResult<(i32, i32)> {
    // "x:123 y:456 screen:0 window:123456"
    let mut x = 0i32;
    let mut y = 0i32;
    for part in s.split_whitespace() {
        if part.starts_with("x:") {
            x = part[2..].parse().unwrap_or(0);
        } else if part.starts_with("y:") {
            y = part[2..].parse().unwrap_or(0);
        }
    }
    Ok((x, y))
}

fn parse_xrandr_line(line: &str) -> Option<Monitor> {
    // "HDMI-1 connected primary 1920x1080+0+0 (normal left inverted ...) 597mm x 336mm"
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 || !parts[1].contains("connected") {
        return None;
    }
    let name = parts[0].to_string();
    // Find the geometry part (e.g., "1920x1080+0+0")
    let geom = parts.iter().find(|p| p.contains("x") && p.contains("+"))?;
    let (size, offset) = geom.split_once("+")?;
    let (w_str, h_str) = size.split_once("x")?;
    let width = w_str.parse().ok()?;
    let height = h_str.parse().ok()?;
    let offset_parts: Vec<&str> = offset.split("+").collect();
    if offset_parts.len() != 2 {
        return None;
    }
    let x = offset_parts[0].parse().ok()?;
    let y = offset_parts[1].parse().ok()?;
    Some(Monitor {
        name,
        geometry: Rect::new(x, y, width, height),
        scale: 1.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_xdotool_pos_works() {
        let (x, y) = parse_xdotool_pos("x:123 y:456 screen:0 window:123456").unwrap();
        assert_eq!(x, 123);
        assert_eq!(y, 456);
    }

    #[test]
    fn parse_xrandr_line_works() {
        let line = "HDMI-1 connected primary 1920x1080+0+0 (normal left inverted ...) 597mm x 336mm";
        let mon = parse_xrandr_line(line).unwrap();
        assert_eq!(mon.name, "HDMI-1");
        assert_eq!(mon.geometry.x, 0);
        assert_eq!(mon.geometry.y, 0);
        assert_eq!(mon.geometry.width, 1920);
        assert_eq!(mon.geometry.height, 1080);
    }
}