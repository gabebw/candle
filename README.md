# Candle :candle:

Candle lets you use CSS selectors to slice and dice any HTML on the command
line. [See below](#usage) for examples.

Since Candle uses Firefox's real-world browser engine, you can pass it any HTML
or CSS you can think of.

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

    curl https://daringfireball.net | candle 'dl a:not([title]) {text}'

    Jack Dorsey’s Twitter Account Was Compromised
    NetNewsWire 5.0

We can show the `href` attribute instead:

    curl https://daringfireball.net | candle 'dl a:not([title]) attr{href}'

    https://techcrunch.com/2019/08/30/someone-hacked-jack-dorseys-own-twitter-account/
    https://inessential.com/2019/08/26/netnewswire_5_0_now_available

Or we can show both the text and the `href`:

    curl https://daringfireball.net | candle 'dl a:not([title]) attr{text}, dl a:not([title]) {href}'

    Jack Dorsey’s Twitter Account Was Compromised
    https://techcrunch.com/2019/08/30/someone-hacked-jack-dorseys-own-twitter-account/
    NetNewsWire 5.0
    https://inessential.com/2019/08/26/netnewswire_5_0_now_available

## Inspiration

The idea for this as well as the syntax was taken from
https://github.com/ericchiang/pup.
