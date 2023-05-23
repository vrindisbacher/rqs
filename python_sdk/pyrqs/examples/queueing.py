from pyrqs.core import Queue, Message, MessageHandler


def produce(message_handler):
    for i in range(10):
        messages = []
        for j in range(10):
            id = f"message-{i}-{j}"
            print("produced:", id)
            message = Message(id, "hello")
            messages.append(message)
        message_handler.produce(messages)


def consume(message_handler):
    while True:
        messages = message_handler.consume()
        if messages == []:
            break
        for message in messages:
            print("received:", message.message_id)
            message.delete()


def create_queue(base_url, queue_id, read_timeout, max_batch):
    queue = Queue(base_url, queue_id, read_timeout, max_batch)
    queues = queue.list().get_data()
    if queue_id not in queues:
        queue.create()
    return queue


def main():
    base_url = "http://127.0.0.1:8080"
    queue_id = "my-queue"
    read_timeout = 10
    max_batch = 10

    # create a queue with the config above
    queue = create_queue(base_url, queue_id, read_timeout, max_batch)

    # spin up a message handler to produce and consume from the associated queue
    message_handler = MessageHandler(base_url, queue)
    produce(message_handler)
    consume(message_handler)


if __name__ == "__main__":
    main()
