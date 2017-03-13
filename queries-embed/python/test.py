from snips_queries import SnipsQueries

q=SnipsQueries("../../data", ["BookRestaurant"])

r= q.run_intent_classification("book me a restaurant for five people tonight", 0.4)
print("%s" % r) 

r= q.run_tokens_classification("book me a restaurant for five people tonight", "BookRestaurant")
print("%s" % r) 
