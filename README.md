# Candle

Shine a little light on your HTML.

## Installation

    cargo install --git https://github.com/gabebw/candle

The binary is called `candle`.

## Usage

You should pipe HTML to `candle`, then tell it what to do with it using CSS.

Because candle uses Firefox's real-world CSS parsing engine, you can pass it
arbitrarily-complex CSS selectors, just like you'd write in real CSS.

Let's show the inner text for some elements:

    curl https://daringfireball.net | candle 'dl.linkedlist dt a:not([title]) {text}'

    Jack Dorsey’s Twitter Account Was Compromised
    NetNewsWire 5.0
    Filmmaker Mode (a.k.a. Death to Motion Smoothing)
    Apple Expands Third-Party Repair Program
    Apple Sends Invitations for September 10 Event

The `{text}` at the end means "show me the inner text for what was selected".

Let's show the `href` attribute instead:

    curl https://daringfireball.net | candle 'dl.linkedlist dt a:not([title]) attr{href}'

    https://techcrunch.com/2019/08/30/someone-hacked-jack-dorseys-own-twitter-account/
    https://inessential.com/2019/08/26/netnewswire_5_0_now_available
    https://www.experienceuhd.com/filmmakermode
    https://www.apple.com/newsroom/2019/08/apple-offers-customers-even-more-options-for-safe-reliable-repairs/
    https://www.loopinsight.com/2019/08/29/apple-sends-invite-for-september-10-event/

## Inspiration

The idea for this as well as the syntax was taken from
https://github.com/ericchiang/pup, which sadly no longer compiles.
