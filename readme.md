## RQS - Rust Queueing Service

The following repo implements a simple EDI service. There are multiple paradigms supported by the service: 
1. One to One Messaging: Where one producer sends messages to one consumer.
2. Worker Distribution: Where producer(s) send messages to a queue and multiple workers split up the work of processing messages. 
3. One to Many Messaging: Where one producer sends a message to multiple consumers. 
4. Many to Many Messaging: Where producers send messages to multiple consumers.

## Development 
The project is set up on the wonderful actix-web framework. The main entry point is in `src/main.rs`. `*_api.rs` implement the respective endpoints for a service. For example, `message_api.rs` implements all the endpoints for `message` specific actions. 

## Running 
Simply clone the repo and using your terminal run `cargo run`. 

## The Service 

The RQS (Rust Queueing Service) has three key components: Queues, Messages, and Exchanges.

### Messages
Messages are the entities that are sent to queues and received by consumers. They contain the actual content to be processed.

### Queues 
Queues are logical entities that receive messages and pass them to consumers upon request. They act as a buffer between the sender and receiver. Two important configurations of queues are:
- `readTimeout`: After a message is read from a queue, it is temporarily hidden for the duration of the readTimeout. During this time, the consumer has the opportunity to process and remove the message from the queue. If the consumer doesn't remove the message within the timeout period, the message becomes visible again and can be read by the same or another consumer.
- `maxBatch`: The maxBatch parameter determines the maximum number of messages that a queue can provide to a consumer in a single request or batch.

### Exchanges 

Exchanges are routing mechanisms within the RQS. They are associated with queues specified by the user. There are two types of exchanges:

- `Fanout`: A fanout exchange multicasts messages to all of its bound queues. This means that every queue bound to the exchange will receive a copy of each message sent to the exchange.
- `Id`: An ID exchange selects the destination queue for a message based on matching the message ID and queue IDs of its bound queues. Each message is routed to the queue that has a matching ID with the message, ensuring that the message is delivered to the appropriate destination.

These components work together to facilitate reliable message delivery and processing within the RQS system.

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
- POST `/exchange/new`: creates a new exchange based on the `exchangeType` 
   - Request Body
    ```json 
    {
        "exchangeId": number - how many seconds to hide message after reading, 
        "queueIds": a list of bound queues,
        "exchangeType": a string literal - either FANOUT or ID 
    }
    ```
   - Response 
    ```json 
    {
        "data": a string with the new exchange's uuid, 
        "error": an error if any 
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
    ```
- GET `/exchange/list`: lists all exchanges 
    - Response
    ```json 
    {
        "data": a list of exchange ids, 
        "error": an error if any 
    }
    ```
- POST `/exchange/add`: adds message to queue(s) by routing it through the exchange
    - Request Body 
    ```json 
    {
        "exchangeId": string, 
        "messages": {
            messageId: string,
            content: string 
        }[]
    }
    ```
    - Response 
    ```json 
    {
        "data": string list representing the new message uuids - note you can't do anything with these,
        "error": an error if any 
    }
    ```

## Examples
Please see `python_sdk/pyrqs/examples` for example of each possible exchange / queue set up. 