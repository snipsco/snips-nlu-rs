Snips NLU Rust
==============

.. image:: https://travis-ci.org/snipsco/snips-nlu-rs.svg?branch=develop
   :target: https://travis-ci.org/snipsco/snips-nlu-rs

Installation
------------

Add it to your ``Cargo.toml``:

.. code-block:: toml

   [dependencies]
   snips-nlu-lib = { git = "https://github.com/snipsco/snips-nlu-rs", branch = "master" }

Add ``extern crate snips_nlu_lib`` to your crate root and you are good to go!


Intent Parsing with Snips NLU
-----------------------------

The purpose of the main crate of this repository, ``snips-nlu-lib``, is to perform an information
extraction task called *intent parsing*.

Let’s take an example to illustrate the main purpose of this lib, and consider the following sentence:

.. code-block:: text

   "What will be the weather in paris at 9pm?"

Properly trained, the Snips NLU engine will be able to extract structured data such as:

.. code-block:: json

   {
      "intent": {
         "intentName": "searchWeatherForecast",
         "probability": 0.95
      },
      "slots": [
         {
            "value": "paris",
            "entity": "locality",
            "slotName": "forecast_locality"
         },
         {
            "value": {
               "kind": "InstantTime",
               "value": "2018-02-08 20:00:00 +00:00"
            },
            "entity": "snips/datetime",
            "slotName": "forecast_start_datetime"
         }
      ]
   }


In order to achieve such a result, the NLU engine needs to be fed with a trained model (json file).
This repository only contains the inference part, in order to produce trained models please check
the `Snips NLU python library <https://github.com/snipsco/snips-nlu>`_.


API Usage
---------

Check out the `rust example <snips-nlu-lib/examples>`_ to see how to use Snips.
Here is how you can run the example:

.. code-block:: bash

   cd snips-nlu-lib
   cargo run --example weather examples/trained_assistant.json "What will be the weather in London tomorrow at 8am?"


License
-------

Licensed under either of
 * Apache License, Version 2.0 (`LICENSE-APACHE <LICENSE-APACHE>`_ or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license (`LICENSE-MIT <LICENSE-MIT>`_) or http://opensource.org/licenses/MIT)
at your option.

Contribution
------------

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
