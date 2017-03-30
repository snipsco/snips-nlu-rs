from snips_queries import IntentParser

#q=IntentParser(data_binary=bytearray(open("/home/fredszaq/Work/tmp/builtins_final.zip", "rb").read()))
q=IntentParser(data_path="/home/fredszaq/Work/tmp/builtins_final")

r= q.get_intent("book me a restaurant for five people tonight")
print("%s" % r) 

r= q.get_entities("book me a restaurant for five people tonight", "BookRestaurant")
print("%s" % r) 
