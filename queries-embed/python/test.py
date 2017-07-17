# coding=utf-8
from __future__ import absolute_import
from __future__ import division
from __future__ import print_function
from __future__ import unicode_literals

from snips_queries import IntentParser

parser = IntentParser("en", data_path="../../data/untracked")

text = "Make me two hot cups of tea please"

result = parser.parse(text)
print(result)
