use futures::executor::block_on;

use futures::{channel::mpsc, Stream, StreamExt};
use windows::{
    Foundation::{EventRegistrationToken, TypedEventHandler},
    UI::ViewManagement::{UIColorType, UISettings},
};

#[derive(Debug)]
pub enum Error {
    WinError(windows::core::Error),
}
pub struct Receiver {
    settings: UISettings,
    token: EventRegistrationToken,
    rx: mpsc::Receiver<ColorScheme>,
}

impl Receiver {
    pub fn try_new() -> Result<Self, Error> {
        let settings = UISettings::new().map_err(Error::WinError)?;
        let (mut tx, rx) = mpsc::channel(1);
        let token = settings
            .ColorValuesChanged(&TypedEventHandler::new(
                move |settings: &Option<UISettings>, _| {
                    let Some(settings) = settings else {
                        return Ok(());
                    };
                    settings
                        .GetColorValue(UIColorType::Background)
                        .map(|color| {
                            tx.try_send(ColorScheme::from(color)).unwrap_or_else(|err| {
                                if err.is_full() {
                                    todo!()
                                }
                            })
                        })
                },
            ))
            .map_err(Error::WinError)?;
        Ok(Receiver {
            settings,
            token,
            rx,
        })
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        self.settings
            .RemoveColorValuesChanged(self.token)
            .unwrap_or(())
    }
}

impl Stream for Receiver {
    type Item = ColorScheme;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_next_unpin(cx)
    }
}

// Todo: Implement Unpin, FusedStream and Debug

#[derive(Debug)]
pub enum ColorScheme {
    Dark,
    Light,
}

impl From<windows::UI::Color> for ColorScheme {
    fn from(color: windows::UI::Color) -> Self {
        if 16 * (color.G as u16) + 11 * (color.R as u16) + 5 * (color.B as u16) < 32 * 192 {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        }
    }
}

fn main() {
    block_on(async {
        let mut rx = Receiver::try_new().unwrap();
        while let Some(x) = rx.next().await {
            println!("Scheme: {x:?}");
        }
    })
}
