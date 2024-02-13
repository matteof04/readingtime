# Readingtime

 Reading time calculator bot for Telegram

## Setup

Before program execution set the BOT_TOKEN environment variable with the token provided by BotFather.\
By default, the LOG_LEVEL environment variable is set to INFO.\
If you want extremly verbose and detailed output, you can set it to TRACE or DEBUG.\
If you want less output you can set it to WARN, or for even less output, to ERROR, although this should be avoided.\
You can also set the word per minute count using the WPM environment variable. The default is 225.

## Usage

Send a message with an URL or a text link and, after it has fetched and parsed the content, will respond to your message with the estimated reading time.
