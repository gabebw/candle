# Candle

Candle lets you use CSS selectors to slice and dice any HTML on the command
line. Just pipe in any HTML and tell Candle what to do with it.

Because candle uses Firefox's real-world CSS parsing engine, you can pass it
arbitrarily-complex CSS selectors, just like you'd write in real CSS.

## Installation

On a Mac using Homebrew (Rust not required):

    brew install gabebw/formulae/candle

Or to build from source (requires the Rust package manager,
[Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)):

    cargo install --git https://github.com/gabebw/candle

The binary is called `candle`.

## Usage

Let's start small:

    echo "<h1 class='bar'>foo <span>and foo</span></h1>" | candle 'h1 {text}'
    foo and foo

The `{text}` at the end of the selector means "show me the inner text for what was selected".

Let's get an attribute:

    echo "<h1 class='bar'>foo <span>and foo</span></h1>" | candle 'h1 attr{class}'
    bar

To get an attribute, use `attr{ATTRIBUTE_NAME}`.

Now let's parse a real webpage:

    curl https://daringfireball.net | candle 'dl.linkedlist dt a:not([title]) {text}'

    Jack Dorsey’s Twitter Account Was Compromised
    NetNewsWire 5.0
    Filmmaker Mode (a.k.a. Death to Motion Smoothing)
    Apple Expands Third-Party Repair Program
    Apple Sends Invitations for September 10 Event

We can show the `href` attribute instead:

    curl https://daringfireball.net | candle 'dl.linkedlist dt a:not([title]) attr{href}'

    https://techcrunch.com/2019/08/30/someone-hacked-jack-dorseys-own-twitter-account/
    https://inessential.com/2019/08/26/netnewswire_5_0_now_available
    https://www.experienceuhd.com/filmmakermode
    https://www.apple.com/newsroom/2019/08/apple-offers-customers-even-more-options-for-safe-reliable-repairs/
    https://www.loopinsight.com/2019/08/29/apple-sends-invite-for-september-10-event/

## Inspiration

The idea for this as well as the syntax was taken from
https://github.com/ericchiang/pup.
