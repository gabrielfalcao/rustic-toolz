use console::style;
use indicatif::{ProgressBar, ProgressStyle};

pub fn template_bar<'a>(size: u64, template: &'a str) -> ProgressBar {
    let progress_bar = ProgressBar::new(size);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(style(template).color256(199).to_string().as_str())
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );
    progress_bar
}

pub fn default_bar<'a>(size: u64) -> ProgressBar {
    template_bar(
        size,
        "{spinner:.bold.199} {prefix:.199}▕{bar:.199}▏{percent}% {msg:.bold.199}",
    )
}
