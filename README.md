# Candle :candle:

Candle lets you use CSS selectors to slice and dice any HTML on the command
line. It can also pretty-print any HTML. [See below](#usage) for examples.

Since Candle uses Firefox's real-world browser engine, you can pass it any HTML
or CSS you can think of.

Candle is written in Rust, and is very fast: almost any combination of HTML and
CSS you can throw at it will complete in <50ms.

## Installation

On a Mac using Homebrew (Rust not required):

    brew install gabebw/formulae/candle

Or to build from source (requires the Rust package manager,
[Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)):

    cargo install --git https://github.com/gabebw/candle

The binary is called `candle`.

## Usage

Let's start by getting the text inside an element:

    $ echo "<h1 id='cool' class='bar'>foo <span>and foo</span></h1>" | candle 'h1 {text}'
    foo and foo

The `{text}` at the end of the selector means "show me the inner text for what was selected".

Note that candle expects you to pipe it the HTML that it will process.

Let's get an attribute:

    $ echo "<h1 id='cool' class='bar'>foo <span>and foo</span></h1>" | candle 'h1 attr{class}'
    bar

To get an attribute, use `attr{ATTRIBUTE_NAME}`.

Let's print out some HTML:

    $ echo "<h1 id='cool' class='bar'>foo <span>and foo</span></h1>" | candle 'h1 {html}'
    <h1 class="bar" id="cool">
      foo
      <span>
        and foo
      </span>
    </h1>

Note that the HTML is pretty-printed for you. Attributes can be shown in any
order, regardless of their original order in the input.

By printing out HTML, you can pipe `candle` output to `candle` again, and build
up a chain of operations:

    $ echo "<h1 id='cool' class='bar'>foo <span>and foo</span></h1>" | \
      candle 'h1 {html}' | \
      candle 'span {text}'
    and foo

In this case, `candle 'span {text}'` would get the same result without piping,
but the `{html}` filter can be helpful when you're not sure what the HTML looks
like and want to poke at it.

Now let's parse a real webpage:

    $ curl https://daringfireball.net | candle '.article h1 a {text}'
    Apple Addresses Siri Privacy Protections
    Siri, Privacy, and Trust
    Superhuman and Email Privacy

We can show the `href` attribute instead:

    $ curl https://daringfireball.net | candle '.article h1 a attr{href}'
    https://daringfireball.net/2019/08/apple_siri_privacy
    https://daringfireball.net/2019/08/siri_privacy_trust
    https://daringfireball.net/2019/07/superhuman_and_email_privacy

Or we can show both the text and the `href`:

    $ curl https://daringfireball.net | candle '.article h1 a {text}, .article h1 a attr{href}'
    Apple Addresses Siri Privacy Protections
    https://daringfireball.net/2019/08/apple_siri_privacy
    Siri, Privacy, and Trust
    https://daringfireball.net/2019/08/siri_privacy_trust
    Superhuman and Email Privacy
    https://daringfireball.net/2019/07/superhuman_and_email_privacy

To format the HTML prettily, call it without arguments:

    $ curl https://daringfireball.net | candle
    <html lang="en" class="daringfireball-net">
      <head>
        <meta charset="UTF-8"></meta>
        <title>
          Daring Fireball
        </title>
        <meta name="viewport" content="width=500, minimum-scale=0.45"></meta>
        <link rel="apple-touch-icon-precomposed" href="/graphics/apple-touch-icon.png"></link>
        <link rel="shortcut icon" href="/graphics/favicon.ico?v=005"></link>
        <link rel="mask-icon" color="#4a525a" href="/graphics/dfstar.svg"></link>

    ...and so on...

(This is exactly equivalent to calling `candle 'html {html}'`.)

## Inspiration

The idea for this as well as the syntax was taken from
https://github.com/ericchiang/pup.
