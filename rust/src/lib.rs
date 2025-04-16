#![feature(thread_local)]
#![allow(static_mut_refs)]

use anyhow::anyhow;
use jsi::de::JsiDeserializeError;
use jsi::{AsValue, JsiArray, JsiFn, JsiObject, JsiValue, PropName, RuntimeClone};
use jsi::{FromValue, RuntimeHandle};
use linkify::LinkFinder;
use ordered_float::NotNan;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde::de::Error;
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::mem::MaybeUninit;
use std::sync::{LazyLock, Mutex};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "ios")]
mod ios;

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

static mut STYLE_CACHE: MaybeUninit<HashMap<TextStyle, JsiValue<'static>>> = MaybeUninit::uninit();

static mut STRING_CACHE: MaybeUninit<HashMap<String, JsiValue<'static>>> = MaybeUninit::uninit();
static mut PROP_NAME_CACHE: MaybeUninit<HashMap<String, PropName<'static>>> = MaybeUninit::uninit();
static mut COLOR_CACHE: MaybeUninit<HashMap<[u8; 4], JsiValue<'static>>> = MaybeUninit::uninit();
static mut FONT_STYLE_CACHE: MaybeUninit<HashMap<FontStyle, JsiValue<'static>>> =
    MaybeUninit::uninit();
static mut FONT_FAMILIES_CACHE: MaybeUninit<HashMap<Vec<Cow<'static, str>>, JsiValue<'static>>> =
    MaybeUninit::uninit();
static mut F32_CONSTRUCTOR: MaybeUninit<JsiFn<'static>> = MaybeUninit::uninit();

static mut LINKIFY: MaybeUninit<LinkFinder> = MaybeUninit::uninit();

static MUTEX: Mutex<()> = Mutex::new(());

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FontWeight {
    Invisible = 0,
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
    ExtraBlack = 1000,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FontWidth {
    UltraCondensed = 1,
    ExtraCondensed = 2,
    Condensed = 3,
    SemiCondensed = 4,
    Normal = 5,
    SemiExpanded = 6,
    Expanded = 7,
    ExtraExpanded = 8,
    UltraExpanded = 9,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FontSlant {
    Upright,
    Italic,
    Oblique,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TextDecoration {
    NoDecoration = 0,
    Underline = 1,
    Overline = 2,
    LineThrough = 4,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FontStyle {
    weight: FontWeight,
    width: FontWidth,
    slant: FontSlant,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextStyle {
    font_size: Option<NotNan<f64>>,

    font_style: Option<FontStyle>,

    decoration: Option<TextDecoration>,

    font_families: Option<Vec<Cow<'static, str>>>,

    color: Option<[u8; 4]>,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle::default_with_size(18.0)
    }
}

impl TextStyle {
    fn default_with_size(size: f64) -> Self {
        Self {
            font_size: Some(NotNan::new(size).unwrap()),
            font_style: None,
            decoration: None,
            font_families: None,
            color: None,
        }
    }
}

impl Default for FontWeight {
    fn default() -> Self {
        FontWeight::Normal
    }
}

impl Default for FontWidth {
    fn default() -> Self {
        FontWidth::Normal
    }
}

impl Default for FontSlant {
    fn default() -> Self {
        FontSlant::Upright
    }
}

impl Default for FontStyle {
    fn default() -> Self {
        Self {
            weight: FontWeight::Normal,
            width: FontWidth::Normal,
            slant: FontSlant::Upright,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextSegment {
    content: String,
    style: TextStyle,

    href: Option<String>,
}

#[derive(Clone, Debug)]
pub struct MarkdownOptions {
    h1_font_size: f64,
    h2_font_size: f64,
    h3_font_size: f64,
    h4_font_size: f64,
    h5_font_size: f64,
    h6_font_size: f64,
    base_font_size: f64,
    link_color: [u8; 4],
    height_multiplier: f64,
    font_families: Option<Vec<Cow<'static, str>>>,
    code_block_font_family: Cow<'static, str>,
    theme: Cow<'static, str>,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            h1_font_size: 46.8,
            h2_font_size: 39.6,
            h3_font_size: 32.4,
            h4_font_size: 23.4,
            h5_font_size: 18.0,
            h6_font_size: 15.3,
            base_font_size: 18.0,
            link_color: [0, 122, 255, 255],
            height_multiplier: 1.0,
            font_families: None,
            code_block_font_family: Cow::Borrowed("monospace"),
            theme: Cow::Borrowed("base16-ocean.dark"),
        }
    }
}

pub fn get_number<'rt>(value: JsiValue<'rt>, rt: &mut RuntimeHandle<'rt>) -> anyhow::Result<f64> {
    if !value.is_number() {
        return Err(anyhow!("Expected a number"));
    }

    Ok(f64::from_value(&value, rt).ok_or(JsiDeserializeError::custom("Expected a number"))?)
}

impl<'a> FromValue<'a> for MarkdownOptions {
    fn from_value(value: &JsiValue<'a>, rt: &mut RuntimeHandle<'a>) -> Option<Self> {
        let mut base_font_size = 18.0;
        let mut h1_font_size = 46.8;
        let mut h2_font_size = 39.6;
        let mut h3_font_size = 32.4;
        let mut h4_font_size = 23.4;
        let mut h5_font_size = 18.0;
        let mut h6_font_size = 15.3;
        let mut link_color: [u8; 4] = [0, 122, 255, 255];
        let mut height_multiplier = 1.0;
        let mut font_families = None;
        let mut code_block_font_family = Cow::from("monospace");
        let mut theme = Cow::from("base16-ocean.dark");

        if value.is_object() {
            let obj = JsiObject::from_value(&value, rt)?;

            base_font_size =
                get_number(obj.get(get_prop_name(rt, "base_font_size"), rt), rt).unwrap_or(18.0);

            h1_font_size = base_font_size * 2.6;
            h2_font_size = base_font_size * 2.2;
            h3_font_size = base_font_size * 1.8;
            h4_font_size = base_font_size * 1.3;
            h5_font_size = base_font_size * 1.0;
            h6_font_size = base_font_size * 0.85;

            if let Ok(val) = get_number(obj.get(get_prop_name(rt, "h1_font_size"), rt), rt) {
                h1_font_size = val;
            }

            if let Ok(val) = get_number(obj.get(get_prop_name(rt, "h2_font_size"), rt), rt) {
                h2_font_size = val;
            }

            if let Ok(val) = get_number(obj.get(get_prop_name(rt, "h3_font_size"), rt), rt) {
                h3_font_size = val;
            }

            if let Ok(val) = get_number(obj.get(get_prop_name(rt, "h4_font_size"), rt), rt) {
                h4_font_size = val;
            }

            if let Ok(val) = get_number(obj.get(get_prop_name(rt, "h5_font_size"), rt), rt) {
                h5_font_size = val;
            }

            if let Ok(val) = get_number(obj.get(get_prop_name(rt, "h6_font_size"), rt), rt) {
                h6_font_size = val;
            }

            let link_color_prop = obj.get(get_prop_name(rt, "link_color"), rt);

            if link_color_prop.is_object() {
                let arr = JsiArray::from_value(&link_color_prop, rt).unwrap();
                let obj = JsiObject::from_value(&link_color_prop, rt).unwrap();

                if arr.len(rt) != 4 {
                    return None;
                }

                for i in 0..arr.len(rt) {
                    let val = get_number(obj.get(get_prop_name(rt, &i.to_string()), rt), rt)
                        .unwrap_or(0.0);
                    link_color[i] = val as u8;
                }
            }

            if let Ok(val) = get_number(obj.get(get_prop_name(rt, "height_multiplier"), rt), rt) {
                height_multiplier = val;
            }

            let val = obj.get(get_prop_name(rt, "font_families"), rt);

            if val.is_object() {
                let arr = JsiArray::from_value(&val, rt).unwrap();
                let obj = JsiObject::from_value(&val, rt).unwrap();

                let mut families = Vec::new();
                for i in 0..arr.len(rt) {
                    let family =
                        String::from_value(&obj.get(get_prop_name(rt, &i.to_string()), rt), rt)
                            .unwrap();
                    families.push(Cow::Owned(family));
                }
                font_families = Some(families);
            }

            let code_block_font_family_prop =
                obj.get(get_prop_name(rt, "code_block_font_family"), rt);
            if code_block_font_family_prop.is_string() {
                code_block_font_family =
                    String::from_value(&code_block_font_family_prop, rt)?.into();
            }

            let theme_prop = obj.get(get_prop_name(rt, "theme"), rt);
            if theme_prop.is_string() {
                theme = String::from_value(&theme_prop, rt)?.into();
            }
        }

        Some(Self {
            base_font_size,
            h1_font_size,
            h2_font_size,
            h3_font_size,
            h4_font_size,
            h5_font_size,
            h6_font_size,
            link_color,
            height_multiplier,
            font_families,
            code_block_font_family,
            theme,
        })
    }
}

#[cfg(test)]
mod test;

pub fn init(rt: *mut jsi::sys::Runtime) {
    let mut rt = RuntimeHandle::new_unchecked(rt);

    let mut global = rt.global();

    let markdown = JsiFn::from_host_fn(
        &PropName::new("JsiParseMarkdown", &mut rt),
        2,
        Box::new(move |_this, args, rt| {
            let markdown_input = std::string::String::from_value(args.get(0).unwrap(), rt).unwrap();
            let markdown_options = match args.get(1) {
                Some(val) => MarkdownOptions::from_value(val, rt).unwrap_or_default(),
                None => MarkdownOptions::default(),
            };

            let segments = parse_markdown(&markdown_input, &markdown_options);
            let arr = JsiArray::new(segments.len(), rt).as_value(rt);

            let mut obj =
                JsiObject::from_value(&arr, rt).ok_or(anyhow::anyhow!("Failed to create array"))?;

            let _lock = MUTEX.lock().unwrap();

            for (i, segment) in segments.iter().enumerate() {
                let val = textsegment_to_jsi_value(rt, segment, &markdown_options);
                obj.set(PropName::new(&i.to_string(), rt), &val, rt);
            }

            Ok(arr)
        }),
        &mut rt,
    );

    global.set(
        PropName::new("JsiParseMarkdown", &mut rt),
        &markdown.as_value(&mut rt),
        &mut rt,
    );

    unsafe {
        STYLE_CACHE.as_mut_ptr().write(HashMap::new());
        STRING_CACHE.as_mut_ptr().write(HashMap::new());
        PROP_NAME_CACHE.as_mut_ptr().write(HashMap::new());
        COLOR_CACHE.as_mut_ptr().write(HashMap::new());
        FONT_STYLE_CACHE.as_mut_ptr().write(HashMap::new());
        FONT_FAMILIES_CACHE.as_mut_ptr().write(HashMap::new());
        let mut linkify = LinkFinder::new();
        linkify.url_can_be_iri(false);
        linkify.url_must_have_scheme(true);
        linkify.kinds(&[linkify::LinkKind::Url]);
        LINKIFY.as_mut_ptr().write(linkify);

        let f32 = JsiFn::from_value(
            &global.get(PropName::new("Float32Array", &mut rt), &mut rt),
            &mut rt,
        );

        if let Some(f32) = f32 {
            F32_CONSTRUCTOR.as_mut_ptr().write(f32);
        }
    }
}

pub fn parse_markdown(markdown_input: &str, opts: &MarkdownOptions) -> Vec<TextSegment> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let linkify = unsafe { &mut *LINKIFY.as_mut_ptr() };
    let links = linkify.links(markdown_input);

    let mut new_markdown_input = String::with_capacity(markdown_input.len());
    let mut last_pos = 0;
    for link in links {
        if !link.as_str().starts_with("https://") && !link.as_str().starts_with("http://") {
            continue;
        }

        let start = link.start();
        let end = link.end();
        new_markdown_input.push_str(&markdown_input[last_pos..start]);
        new_markdown_input.push('<');
        new_markdown_input.push_str(link.as_str());
        new_markdown_input.push('>');
        last_pos = end;
    }

    new_markdown_input.push_str(&markdown_input[last_pos..]);
    let new_markdown_input = new_markdown_input.replace('\n', "\\\n");

    let parser = Parser::new_ext(new_markdown_input.as_str(), options);

    let mut segments = Vec::new();
    let mut current_styles: Vec<TextStyle> = Vec::new();
    let mut link_href: Option<String> = None;
    let mut pending_breaks = String::new();
    let mut in_code_block = false;
    let mut code_lang = None;
    let mut code_block_buffer = VecDeque::new();

    for event in parser {
        match event {
            Event::Start(tag) => {
                let mut new_style = current_styles.last().cloned().unwrap_or_else(|| {
                    let mut style = TextStyle::default_with_size(opts.base_font_size);
                    style.font_families = opts.font_families.clone();

                    style
                });
                match tag {
                    Tag::Strong => {
                        if new_style.font_style.is_none() {
                            new_style.font_style = Some(FontStyle::default());
                        }
                        new_style.font_style.as_mut().unwrap().weight = FontWeight::Bold;
                    }
                    Tag::Emphasis => {
                        if new_style.font_style.is_none() {
                            new_style.font_style = Some(FontStyle::default());
                        }
                        new_style.font_style.as_mut().unwrap().slant = FontSlant::Italic;
                    }
                    Tag::Strikethrough => {
                        new_style.decoration = Some(TextDecoration::LineThrough);
                    }
                    Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Fenced(lang)) => {
                        in_code_block = true;
                        code_lang = Some(lang.to_string());
                    }
                    Tag::Heading { level, .. } => {
                        if new_style.font_style.is_none() {
                            new_style.font_style = Some(FontStyle::default());
                        }
                        new_style.font_style.as_mut().unwrap().weight = FontWeight::Bold;
                        new_style.font_size = Some(match level {
                            HeadingLevel::H1 => NotNan::new(opts.h1_font_size).unwrap(),
                            HeadingLevel::H2 => NotNan::new(opts.h2_font_size).unwrap(),
                            HeadingLevel::H3 => NotNan::new(opts.h3_font_size).unwrap(),
                            HeadingLevel::H4 => NotNan::new(opts.h4_font_size).unwrap(),
                            HeadingLevel::H5 => NotNan::new(opts.h5_font_size).unwrap(),
                            HeadingLevel::H6 => NotNan::new(opts.h6_font_size).unwrap(),
                        });
                    }
                    Tag::Link { dest_url, .. } => {
                        link_href = Some(dest_url.to_string());
                        new_style.decoration = Some(TextDecoration::Underline);
                        new_style.color = Some(opts.link_color.clone());
                    }
                    _ => {}
                }
                current_styles.push(new_style);
            }
            Event::End(tag) => {
                match tag {
                    TagEnd::CodeBlock => {
                        in_code_block = false;
                        let code = code_block_buffer.drain(..).collect::<String>();
                        let highlighted_segments =
                            highlight_code_block(&code, code_lang.as_deref(), opts);
                        segments.extend(highlighted_segments);
                        code_lang = None;
                    }
                    TagEnd::Link { .. } => link_href = None,
                    TagEnd::Paragraph => {
                        pending_breaks.push_str("\n\n");
                    }
                    _ => {}
                }
                current_styles.pop();
            }
            Event::Text(text) => {
                if in_code_block {
                    code_block_buffer.push_back(text.to_string());
                } else {
                    let style = current_styles.last().cloned().unwrap_or_default();
                    let content = format!("{}{}", pending_breaks, text);
                    pending_breaks.clear();

                    segments.push(TextSegment {
                        content,
                        style,
                        href: link_href.clone(),
                    });
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_code_block {
                    code_block_buffer.push_back("\n".to_string());
                } else {
                    pending_breaks.push('\n');
                }
            }
            _ => {}
        }
    }

    segments
}

fn highlight_code_block(
    code: &str,
    language: Option<&str>,
    opts: &MarkdownOptions,
) -> Vec<TextSegment> {
    let theme = &THEME_SET.themes.get(opts.theme.as_ref());

    if theme.is_none() {
        return Vec::new();
    }
    let theme = theme.unwrap();

    let syntax = language
        .as_deref()
        .and_then(|lang| SYNTAX_SET.find_syntax_by_token(lang))
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme);

    let mut segments = Vec::new();

    for line in LinesWithEndings::from(code) {
        if let Ok(ranges) = highlighter.highlight_line(line, &SYNTAX_SET) {
            for (style, text) in ranges {
                let color = [
                    style.foreground.r,
                    style.foreground.g,
                    style.foreground.b,
                    255,
                ];

                let text_style = TextStyle {
                    font_size: Some(unsafe { NotNan::new_unchecked(opts.base_font_size) }),
                    font_families: Some(vec![opts.code_block_font_family.clone()]),
                    color: Some(color.into()),
                    ..Default::default()
                };

                segments.push(TextSegment {
                    content: text.to_string(),
                    style: text_style,
                    href: None,
                });
            }
        }
    }

    segments
}

fn get_string(rt: &mut RuntimeHandle<'static>, value: &str) -> JsiValue<'static> {
    let string_cache = unsafe { &mut *STRING_CACHE.as_mut_ptr() };
    if let Some(cached_value) = string_cache.get(value) {
        return (*cached_value).clone(rt);
    }
    let new_value = JsiValue::new_string(value, rt);
    let cloned_value = new_value.clone(rt);

    string_cache.insert(value.to_string(), cloned_value.clone(rt));

    cloned_value
}

fn get_prop_name<'a>(rt: &mut RuntimeHandle<'a>, value: &str) -> PropName<'static> {
    let mut rt = unsafe { RuntimeHandle::new_unchecked(rt.get_inner_mut().get_unchecked_mut()) };

    let prop_name_cache = unsafe { &mut *PROP_NAME_CACHE.as_mut_ptr() };
    if let Some(cached_prop) = prop_name_cache.get(value) {
        return (*cached_prop).clone(&mut rt);
    }

    let new_prop = PropName::new(value, &mut rt);
    prop_name_cache.insert(value.to_string(), new_prop.clone(&mut rt));
    new_prop
}

fn get_color<'a>(rt: &mut RuntimeHandle<'a>, value: [u8; 4]) -> JsiValue<'static> {
    let mut rt = unsafe { RuntimeHandle::new_unchecked(rt.get_inner_mut().get_unchecked_mut()) };

    let color_cache = unsafe { &mut *COLOR_CACHE.as_mut_ptr() };
    if let Some(cached_prop) = color_cache.get(&value) {
        return (*cached_prop).clone(&mut rt);
    }

    let color_arr = unsafe { &mut *F32_CONSTRUCTOR.as_mut_ptr() }
        .call_as_constructor(vec![JsiValue::new_number(4.0)], &mut rt);

    if let Ok(color_arr) = color_arr {
        let mut obj =
            JsiObject::from_value(&color_arr, &mut rt).unwrap_or_else(|| JsiObject::new(&mut rt));

        for (i, val) in value.iter().enumerate() {
            obj.set(
                get_prop_name(&mut rt, &i.to_string()),
                &JsiValue::new_number((*val as f64) / 255.0),
                &mut rt,
            );
        }

        color_cache.insert(value, color_arr.clone(&mut rt));

        return color_arr;
    } else {
        return JsiValue::new_null();
    }
}

fn get_font_style(rt: &mut RuntimeHandle<'static>, style: &FontStyle) -> JsiValue<'static> {
    let font_style_cache = unsafe { &mut *FONT_STYLE_CACHE.as_mut_ptr() };
    if let Some(cached_style) = font_style_cache.get(style) {
        return (*cached_style).clone(rt);
    }

    let mut style_obj = JsiObject::new(rt);
    style_obj.set(
        get_prop_name(rt, "weight"),
        &JsiValue::new_number(style.weight as i32 as f64),
        rt,
    );
    style_obj.set(
        get_prop_name(rt, "width"),
        &JsiValue::new_number(style.width as i32 as f64),
        rt,
    );
    style_obj.set(
        get_prop_name(rt, "slant"),
        &JsiValue::new_number(style.slant as i32 as f64),
        rt,
    );

    let new_value = style_obj.as_value(rt);
    font_style_cache.insert(style.clone(), new_value.clone(rt));
    new_value
}

fn get_font_families(
    rt: &mut RuntimeHandle<'static>,
    families: &Vec<Cow<'static, str>>,
) -> JsiValue<'static> {
    let font_families_cache = unsafe { &mut *FONT_FAMILIES_CACHE.as_mut_ptr() };
    if let Some(cached_value) = font_families_cache.get(families) {
        return (*cached_value).clone(rt);
    }

    let value = JsiArray::new(families.len(), rt).as_value(rt);
    let mut obj = JsiObject::from_value(&value, rt).unwrap_or_else(|| JsiObject::new(rt));

    for (i, family) in families.iter().enumerate() {
        // arr.set(i, &get_string(rt, family), rt);
        obj.set(
            get_prop_name(rt, &i.to_string()),
            &get_string(rt, family),
            rt,
        );
    }

    font_families_cache.insert(families.to_vec(), value.clone(rt));
    value
}

fn get_style(
    rt: &mut RuntimeHandle<'static>,
    style: &TextStyle,
    opts: &MarkdownOptions,
) -> JsiValue<'static> {
    let style_cache = unsafe { &mut *STYLE_CACHE.as_mut_ptr() };

    // Check if the segment is already in the cache
    if let Some(cached_value) = style_cache.get(style) {
        return cached_value.clone(rt);
    }

    let mut obj = JsiObject::new(rt);

    if let Some(font_size) = style.font_size {
        obj.set(
            get_prop_name(rt, "fontSize"),
            &JsiValue::new_number(*font_size),
            rt,
        );
    }

    if let Some(font_style) = &style.font_style {
        obj.set(
            get_prop_name(rt, "fontStyle"),
            &get_font_style(rt, font_style),
            rt,
        );
    }

    if let Some(decoration) = &style.decoration {
        obj.set(
            get_prop_name(rt, "decoration"),
            &JsiValue::new_number(*decoration as i32 as f64),
            rt,
        );
    }

    if let Some(font_families) = &style.font_families {
        obj.set(
            get_prop_name(rt, "fontFamilies"),
            &get_font_families(rt, font_families),
            rt,
        );
    }

    if let Some(color) = &style.color {
        obj.set(get_prop_name(rt, "color"), &get_color(rt, *color), rt);
    }

    if opts.height_multiplier != 1.0 {
        obj.set(
            get_prop_name(rt, "heightMultiplier"),
            &JsiValue::new_number(opts.height_multiplier),
            rt,
        );
    }

    style_cache.insert(style.clone(), obj.as_value(rt));

    obj.as_value(rt)
}

fn textsegment_to_jsi_value(
    rt: &mut RuntimeHandle<'static>,
    segment: &TextSegment,
    opts: &MarkdownOptions,
) -> JsiValue<'static> {
    // Create a new JsiObject for the segment
    let mut object = JsiObject::new(rt);

    object.set(
        get_prop_name(rt, "content"),
        &get_string(rt, &segment.content),
        rt,
    );

    if let Some(href) = &segment.href {
        object.set(get_prop_name(rt, "href"), &get_string(rt, href), rt);
    }

    object.set(
        get_prop_name(rt, "style"),
        &get_style(rt, &segment.style, opts),
        rt,
    );

    let jsi_value = object.as_value(rt);

    jsi_value
}
