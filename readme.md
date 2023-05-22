## Rust EDI 

The following repo implements a simple EDI service. The suggested flow is as follows:

1. There are some queues. 
2. Producer(s) push messages to a queue/queues. 
3. Consumer(s) get messages from a queue/queues. 
4. Consumer(s) acknowledge message and remove them from a queue/queues. 

## Performance 
This is only a toy implementation, but being Rust, it works at baremetal speed. Sending, 

Receiving and Deleting `1000` messages, one at a time, using the Python implementation below takes a mere `2.2` seconds. 

Receiving and Deleting `1000` messages in batches of `10` using the Python implementation below takes a mere `0.9` seconds - a dramatic performance increase over one at a time. 

Of course - network cost should absolutely be taken into account. These were done on localhost, not on a remote server. 

There are many improvements that could be made. For example, web sockets could be used for consumer connections to avoid more than a single request.

## Development 
The project is set up on the wonderful actix-web framework. The main entry point is in `src/main.rs`. `*_api.rs` implement the respective endpoints for a service. For example, `message_api.rs` implements all the endpoints for `message` specific actions. 

## Running 
Simply clone the repo and using your terminal run `cargo run`. 

## Current Endpoints 

- POST `/queue/new`: creates a new queue 
   - Request Body
    ```json 
    {
        "readTimeout": number - how many seconds to hide message after reading, 
        "maxBatch": number - how many messages can be sent to a consumer at once 
        "queueId": string
    }
    ```
   - Response 
    ```json 
    {
        "data": a string with the new queue's uuid, 
        "error": an error if any 
    }
    ```
- GET `/queue/list`: lists all queues 
    - Response
    ```json 
    {
        "data": a list of queue ids, 
        "error": an error if any 
    }
    ```
- POST `/message/new`: adds messages to a specified queue 
    - Request Body 
    ```json 
    {
        "queueId": string, 
        "messages": {
            messageId: string,
            content: string 
        }[]
    }
    ```
    - Response 
    ```json 
    {
        "data": string representing the new message uuids - note you can't do anything with these,
        "error": an error if any 
    }
    ```
- GET `/message/get`: gets a batch of messages - capped at `maxBatch` or the number of message available
    - Response 
    ```json 
    {
        "data" : {
                "messageId": string,
                "content": string,
                "uuid": string
            }[],
        "error": an eror if any 
    }
    ```
- POST `/message/delete`: deletes a message from a queue 
    - Request Body 
    ```json 
    {
        "queueId": string, 
        "messageUuid": string
    }
    ``` 
    - Response 
    ```json 
    {
        "data": a success message, 
        "error": an error if any  
    }
    ```
    **NOTE: Messages can only be deleted from the queue while their read timeout is in effect. Otherwise, they must be read again and deleted within their read timeout.**

## Example 
Here is some python that loosely sends and receives messages with a queue configured with a read timeout of 10 seconds, and a batch size of 10. 

### Sending and Receiving in Batch 
In this case, the code will produce 10 messages until it hits 1000. It will then get 
10 messages at a time, and delete each one until it hits 1000. 

```python 
import requests 
import json 
import datetime 

def create_queue(queue_id, read_timeout, max_batch):
    requests.post("http://127.0.0.1:8080/queue/new", json={"readTimeout": read_timeout, "maxBatch": max_batch, "queueId": queue_id})

def list_queues():
    requests.get("http://127.0.0.1:8080/queue/list")

def publish(queue_id, messages):
    post_data = {
        "queueId": queue_id,
        "messages": messages
    }
    r = requests.post("http://127.0.0.1:8080/message/new", json=post_data)
    print(f"produced {r.json()['data']}")

def delete_message(queue_id, message_uuid): 
    post_data = {
        "queueId": queue_id,
        "messageUuid": message_uuid
    }
    requests.post("http://127.0.0.1:8080/message/new", json=post_data)

def consume_and_delete(queue_id):
    r = requests.get(f"http://127.0.0.1:8080/message/get?queueId={queue_id}")
    response = r.json()
    print(f"received {len(response['data'])} message(s)")
    for message in response['data']:
        uuid = message['uuid']
        print(f"{uuid}")
        post_data = {
            "queueId": queue_id, 
            "messageUuid": uuid 
        }
        r = requests.post("http://127.0.0.1:8080/message/delete", json=post_data)

def main(): 
    start = datetime.datetime.now()
    queue_id = 'my-test-queue'
    create_queue(queue_id, 10, 10)
    list_queues()

    for i in range(100):
        to_publish = []
        for j in range(10):
            to_publish.append({
                "messageId": f"Message {j} in batch {i}",
                "content": json.dumps({"Hello" : "World"})
            })
        publish(queue_id, to_publish)

    for i in range(100):
        consume_and_delete(queue_id)

    print(datetime.datetime.now() - start)


if __name__ == '__main__':
    main()
```

### Sending and Receiving One at a Time
Here is an example of sending and receiving one message at a time. 

```python 
import requests 
import json 
import datetime 

def create_queue(queue_id, read_timeout, max_batch):
    requests.post("http://127.0.0.1:8080/queue/new", json={"readTimeout": read_timeout, "maxBatch": max_batch, "queueId": queue_id})

def list_queues():
    requests.get("http://127.0.0.1:8080/queue/list")

def publish(queue_id, messages):
    post_data = {
        "queueId": queue_id,
        "messages": messages
    }
    r = requests.post("http://127.0.0.1:8080/message/new", json=post_data)
    print(f"produced {r.json()['data']}")

def delete_message(queue_id, message_uuid): 
    post_data = {
        "queueId": queue_id,
        "messageUuid": message_uuid
    }
    requests.post("http://127.0.0.1:8080/message/new", json=post_data)

def consume_and_delete(queue_id):
    r = requests.get(f"http://127.0.0.1:8080/message/get?queueId={queue_id}")
    response = r.json()
    print(f"received {len(response['data'])} message(s)")
    for message in response['data']:
        uuid = message['uuid']
        print(f"{uuid}")
        post_data = {
            "queueId": queue_id, 
            "messageUuid": uuid 
        }
        r = requests.post("http://127.0.0.1:8080/message/delete", json=post_data)

def main(): 
    start = datetime.datetime.now()
    queue_id = 'my-test-queue'
    create_queue(queue_id, 10, 10)
    list_queues()

    for i in range(1000):
        to_publish = [{
                "messageId": f"Message {i}",
                "content": json.dumps({"Hello" : "World"})
            }]
        publish(queue_id, to_publish)
        consume_and_delete(queue_id)

    print(datetime.datetime.now() - start)


if __name__ == '__main__':
    main()
```

In this case, both the producer and consumer send and get 1 message at a time. If the consumer waited for multiple messages to be available, it could get up to 10 messages 
at a time. 
