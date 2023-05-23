from pyrqs.core import Queue, Message, MessageHandler, Exchange, ID


def produce(exchange, message_id):
    for i in range(10):
        messages = []
        for j in range(10):
            print("produced:", message_id)
            # here we produce with a message_id = to the queue we would like to route to
            message = Message(message_id, "hello")
            messages.append(message)
        exchange.produce(messages)


def consume(message_handler):
    while True:
        messages = message_handler.consume()
        if messages == []:
            break
        for message in messages:
            print(
                f"received by queue {message_handler.queue.queue_id}:",
                message.message_id,
            )
            message.delete()


def create_queue(base_url, queue_id, read_timeout, max_batch):
    queue = Queue(base_url, queue_id, read_timeout, max_batch)
    queues = queue.list().get_data()
    if queue_id not in queues:
        queue.create()
    return queue


def create_exchange(base_url, queues, exchange_id, exchange_type):
    exchange = Exchange(base_url, queues, exchange_id, exchange_type)
    if exchange_id not in exchange.list().get_data():
        exchange.create()
    return exchange


def main():
    base_url = "http://127.0.0.1:8080"
    queue_id1 = "my-bound-queue-1"
    queue_id2 = "my-bound-queue-2"
    read_timeout = 10
    max_batch = 10
    base_url = "http://127.0.0.1:8080"
    exchange_id = "my-exchange"
    exchange_type = ID

    # create a queue with the config above
    queue1 = create_queue(base_url, queue_id1, read_timeout, max_batch)
    queue2 = create_queue(base_url, queue_id2, read_timeout, max_batch)

    # create a fanout exchange
    exchange = Exchange(base_url, [queue1, queue2], exchange_id, exchange_type)

    # produce to the exchange - routing messsages to queue1
    message_id = queue1.queue_id
    produce(exchange, message_id)

    # spin up a message handler to consume from our queues
    message_handler1 = MessageHandler(base_url, queue1)
    message_handler2 = MessageHandler(base_url, queue2)
    consume(message_handler1)
    consume(message_handler2)


if __name__ == "__main__":
    main()
