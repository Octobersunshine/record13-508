use crate::models::{ExamPaper, QuestionInPaper, QuestionType};
use std::io::{Cursor, Write};
use zip::{write::FileOptions, ZipWriter, CompressionMethod, result::ZipResult};

fn xml_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&apos;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

fn run(text: &str, bold: bool, size_half_pts: usize, font: &str) -> String {
    let b_tag = if bold { "<w:b/>" } else { "" };
    format!(
        "<w:r><w:rPr>{b_tag}<w:sz w:val=\"{sz}\"/><w:szCs w:val=\"{sz}\"/><w:rFonts w:ascii=\"{f}\" w:hAnsi=\"{f}\" w:eastAsia=\"{f}\" w:cs=\"{f}\"/></w:rPr><w:t xml:space=\"preserve\">{t}</w:t></w:r>",
        b_tag = b_tag,
        sz = size_half_pts,
        f = xml_escape(font),
        t = xml_escape(text),
    )
}

fn run_underline(text: &str, size_half_pts: usize, font: &str) -> String {
    format!(
        "<w:r><w:rPr><w:u w:val=\"single\"/><w:sz w:val=\"{sz}\"/><w:szCs w:val=\"{sz}\"/><w:rFonts w:ascii=\"{f}\" w:hAnsi=\"{f}\" w:eastAsia=\"{f}\" w:cs=\"{f}\"/></w:rPr><w:t xml:space=\"preserve\">{t}</w:t></w:r>",
        sz = size_half_pts,
        f = xml_escape(font),
        t = xml_escape(text),
    )
}

fn paragraph(align: &str, content: &str, spacing_after: usize) -> String {
    let align_attr = if align.is_empty() {
        String::new()
    } else {
        format!("<w:jc w:val=\"{}\"/>", align)
    };
    let spacing = if spacing_after > 0 {
        format!("<w:spacing w:after=\"{}\"/>", spacing_after * 20)
    } else {
        String::new()
    };
    format!(
        "<w:p><w:pPr>{a}{s}</w:pPr>{c}</w:p>",
        a = align_attr,
        s = spacing,
        c = content,
    )
}

fn paragraph_with_indent(indent_left: usize, content: &str) -> String {
    format!(
        "<w:p><w:pPr><w:ind w:left=\"{}\"/></w:pPr>{}</w:p>",
        indent_left,
        content,
    )
}

fn empty_paragraph() -> String {
    "<w:p/>".to_string()
}

fn build_title_section(paper: &ExamPaper) -> String {
    let mut out = String::new();

    out.push_str(&paragraph(
        "center",
        &run(&paper.title, true, 36, "黑体"),
        360,
    ));

    let info_line = format!(
        "总分：{}分    生成时间：{}",
        paper.total_score, paper.created_at
    );
    out.push_str(&paragraph(
        "center",
        &run(&info_line, false, 24, "宋体"),
        120,
    ));

    let header = "姓名：____________    班级：____________    学号：____________    得分：____________";
    out.push_str(&paragraph(
        "center",
        &run(header, false, 24, "宋体"),
        240,
    ));

    out.push_str(&empty_paragraph());
    out
}

fn build_questions_for_type(idx: &mut usize, qs: &[QuestionInPaper], q_type: &QuestionType) -> String {
    let mut out = String::new();
    for q in qs {
        *idx += 1;

        let mut q_runs = String::new();
        q_runs.push_str(&run(&format!("{}. ", *idx), true, 24, "宋体"));
        q_runs.push_str(&run(
            &format!("{}（{}分）", q.content, q.score),
            false,
            24,
            "宋体",
        ));
        out.push_str(&paragraph("", &q_runs, 40));

        match q_type {
            QuestionType::SingleChoice | QuestionType::MultipleChoice => {
                let labels = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];
                for (i, opt) in q.options.iter().enumerate() {
                    let label = labels.get(i).copied().unwrap_or('?');
                    let content = run(&format!("{}. {}", label, opt), false, 24, "宋体");
                    out.push_str(&paragraph_with_indent(480, &content));
                }
            }
            QuestionType::TrueOrFalse => {
                let content = run("（    ）", false, 24, "宋体");
                out.push_str(&paragraph_with_indent(480, &content));
            }
            QuestionType::FillBlank => {
                let content = run_underline("                    ", 24, "宋体");
                out.push_str(&paragraph_with_indent(480, &content));
            }
            QuestionType::ShortAnswer => {
                for _ in 0..5 {
                    let content = run_underline(
                        "                                                            ",
                        24,
                        "宋体",
                    );
                    out.push_str(&paragraph_with_indent(480, &content));
                }
            }
            QuestionType::Essay => {
                for _ in 0..10 {
                    let content = run_underline(
                        "                                                            ",
                        24,
                        "宋体",
                    );
                    out.push_str(&paragraph_with_indent(480, &content));
                }
            }
        }
    }
    out
}

fn build_document_xml(paper: &ExamPaper) -> String {
    let mut body = String::new();
    body.push_str(&build_title_section(paper));

    let mut q_idx = 0usize;
    for section in &paper.sections {
        let title_run = run(&section.label, true, 28, "黑体");
        body.push_str(&paragraph("", &title_run, 80));
        body.push_str(&build_questions_for_type(
            &mut q_idx,
            &section.questions,
            &section.q_type,
        ));
        body.push_str(&empty_paragraph());
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:wpc="http://schemas.microsoft.com/office/word/2010/wordprocessingCanvas"
 xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"
 xmlns:o="urn:schemas-microsoft-com:office:office"
 xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
 xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math"
 xmlns:v="urn:schemas-microsoft-com:vml"
 xmlns:wp14="http://schemas.microsoft.com/office/word/2010/wordprocessingDrawing"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:w10="urn:schemas-microsoft-com:office:word"
 xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"
 xmlns:wpg="http://schemas.microsoft.com/office/word/2010/wordprocessingGroup"
 xmlns:wpi="http://schemas.microsoft.com/office/word/2010/wordprocessingInk"
 xmlns:wne="http://schemas.microsoft.com/office/2006/wordml"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"
 mc:Ignorable="w14 wp14">
<w:body>
{body}
<w:sectPr>
<w:pgSz w:w="11906" w:h="16838"/>
<w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="708" w:footer="708" w:gutter="0"/>
</w:sectPr>
</w:body>
</w:document>"#,
        body = body,
    )
}

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
</Types>"#;

const RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

const DOC_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;

const STYLES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:docDefaults>
<w:rPrDefault><w:rPr><w:rFonts w:ascii="宋体" w:eastAsia="宋体" w:hAnsi="宋体"/><w:sz w:val="24"/><w:szCs w:val="24"/></w:rPr></w:rPrDefault>
</w:docDefaults>
</w:styles>"#;

fn add_file<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    name: &str,
    content: &str,
) -> ZipResult<()> {
    let opts: FileOptions<'_, ()> = FileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.start_file(name, opts)?;
    zip.write_all(content.as_bytes())?;
    Ok(())
}

pub fn paper_to_docx_bytes(paper: &ExamPaper) -> Result<Vec<u8>, String> {
    let doc_xml = build_document_xml(paper);

    let buf = Vec::new();
    let cursor = Cursor::new(buf);
    let mut zip = ZipWriter::new(cursor);

    add_file(&mut zip, "[Content_Types].xml", CONTENT_TYPES)
        .map_err(|e| format!("写入 [Content_Types].xml 失败：{}", e))?;
    add_file(&mut zip, "_rels/.rels", RELS)
        .map_err(|e| format!("写入 _rels/.rels 失败：{}", e))?;
    add_file(&mut zip, "word/document.xml", &doc_xml)
        .map_err(|e| format!("写入 word/document.xml 失败：{}", e))?;
    add_file(&mut zip, "word/_rels/document.xml.rels", DOC_RELS)
        .map_err(|e| format!("写入 word/_rels/document.xml.rels 失败：{}", e))?;
    add_file(&mut zip, "word/styles.xml", STYLES_XML)
        .map_err(|e| format!("写入 word/styles.xml 失败：{}", e))?;

    let cursor = zip.finish().map_err(|e| format!("打包 docx 失败：{}", e))?;
    Ok(cursor.into_inner())
}

pub fn filename_for_paper(paper: &ExamPaper) -> String {
    let safe_title: String = paper
        .title
        .chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect();
    format!("{}_{}.docx", safe_title, &paper.id[..8])
}
