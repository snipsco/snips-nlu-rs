# coding=utf-8
from __future__ import absolute_import
from __future__ import division
from __future__ import print_function
from __future__ import unicode_literals

from snips_queries import IntentParser

q = IntentParser("en", data_path="../../data/untracked")
#q = IntentParser("en", data_zip=bytearray(open("../../data/untracked/builtins_final.zip", "rb").read()))

text = "Book me a table for four people at Le Chalet Savoyard at 9pm"

r = q.get_intent(text, 0.4)
print(r)

r = q.get_entities(text, "BookRestaurant")
print(r)
