## PYRQS

This SDK provides a Python interface for interacting with a RQS. It allows you to create queues, produce and consume messages, and manage exchanges.


## Examples 

Please refer to the examples for usage. 
- `queueing.py` creates a queue, publishes message to the queue, and consumes messages from the queue. 
- `fanout_exchange.py` creates an exchange with two bound queues. It then produces messages through the exchange, and the consumes them from the bound queues. 
- `id_exchange.py` creates an exchange with two bound queues. It then produces messages though the exchange based on ID and consumes the queues. 