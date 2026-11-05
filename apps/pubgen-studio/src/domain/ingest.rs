//! Document ingestion and normalization into Markdown.

use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::Read;
use zip::ZipArchive;

/// Converts DOCX bytes into normalized Markdown text.
pub fn docx_to_md(bytes: &[u8]) -> Result<String, String> {
    let mut archive = ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| format!("Not a valid docx archive: {e}"))?;

    let mut xml_bytes = Vec::new();
    archive
        .by_name("word/document.xml")
        .map_err(|e| format!("Missing word/document.xml: {e}"))?
        .read_to_end(&mut xml_bytes)
        .map_err(|e| format!("Cannot read document.xml: {e}"))?;

    let xml = String::from_utf8_lossy(&xml_bytes).into_owned();
    let mut reader = Reader::from_str(&xml);

    let mut out = String::new();
    let mut para = String::new();
    let mut prefix = String::new();
    let mut in_list = false;
    let mut bold = false;
    let mut italic = false;
    let mut in_text = false;
    let mut buf = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => match e.name().as_ref() {
                b"w:p" => {
                    para.clear();
                    prefix.clear();
                    in_list = false;
                }
                b"w:pStyle" => {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"w:val" {
                            let val = String::from_utf8_lossy(&attr.value);
                            match val.as_ref() {
                                "Heading1" | "Heading 1" | "Judul1" => prefix = "# ".into(),
                                "Heading2" | "Heading 2" | "Judul2" => prefix = "## ".into(),
                                "Heading3" | "Heading 3" | "Judul3" => prefix = "### ".into(),
                                "Title" | "Judul" => prefix = "# ".into(),
                                "Subtitle" | "Subjudul" => prefix = "## ".into(),
                                _ => {}
                            }
                        }
                    }
                }
                b"w:numPr" => {
                    in_list = true;
                }
                b"w:b" => bold = true,
                b"w:i" => italic = true,
                b"w:t" => in_text = true,
                _ => {}
            },
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"w:p" => {
                    let trimmed = para.trim();
                    if !trimmed.is_empty() {
                        if in_list && prefix.is_empty() {
                            out.push_str("- ");
                        } else {
                            out.push_str(&prefix);
                        }
                        out.push_str(trimmed);
                        out.push_str("\n\n");
                    }
                }
                b"w:r" => {
                    bold = false;
                    italic = false;
                }
                b"w:t" => in_text = false,
                _ => {}
            },
            Ok(Event::Text(t)) if in_text => {
                if let Ok(text) = t.decode() {
                    let mut chunk = text.to_string();
                    if bold && italic {
                        chunk = format!("***{chunk}***");
                    } else if bold {
                        chunk = format!("**{chunk}**");
                    } else if italic {
                        chunk = format!("*{chunk}*");
                    }
                    para.push_str(&chunk);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parser error: {e}")),
            _ => {}
        }
    }

    Ok(out.trim().to_string())
}
