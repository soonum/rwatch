# rwatch

The `rwatch` ecosystem is a set of crates that handle data meant to be monitored.

A use case of this ecosystem could be monitoring a garden with various sensors and then trigger actions action based on the data harvested such as start watering the plants or warn user about a possible incoming drought.

## rwatch crates

The crates can be divided into two categories: data aggregator and data producer/consumer which use a connector to interact with an aggregator.

Data aggregator includes the following crates:

+ ['rwatch-core'](rwatch-core): a binary that aggregates data from producers and distribute data wanted by consumers. It can also trigger alarms based on thresholds that are customizable via configuration file.

Data producers/consumers are known as plug-ins, it contains the following crates:

+ ['data-generator'](plugins/data-generator): produce either data based on user-inputs or generate random data. Its main purpose is to help developer to work on `rwatch-core`.
+ ['data-display'](plugins/data-display) : (yet to be implemented) a HTTP server that can display various data or alarms status in a browser.
+ ['physical-sensor'](plugins/physical-sensor): (yet to be implemented) produce data based on value fetched from a physical sensor. It can be used to produce record related to temperature or air moisture. This crate is meant to be embedded in small devices.
