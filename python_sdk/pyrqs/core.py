import requests

FANOUT = "FANOUT"
ID = "ID"


class SuccessResponse:
    """
    Represents a successful response received from an API call.

    Attributes:
        data (Any): The data contained in the response.

    Methods:
        get_data(): Retrieves the data contained in the SuccessResponse object.
    """

    def __init__(self, data):
        assert "data" in data
        self.data = data["data"]

    def get_data(self):
        """
        get_data()

        Retrieves the data contained in the SuccessResponse object.

        Returns:
            Any: The data contained in the SuccessResponse object.
        """
        return self.data


class ErrorResponse(Exception):
    """
    Represents an error response received from an API call.

    Inherits:
        Exception: The base class for all exceptions in Python.

    Attributes:
        None
    """

    ...


class Queue:
    """
    Represents a queue where messages can be produced and consumed.

    Attributes:
        base_url (str): The base URL of the API.
        queue_id (str): The unique identifier for the queue.
        read_timeout (int): The time limit in seconds for which a message can be read from the queue.
        max_batch (int): The maximum number of messages that can be consumed in a batch.

    Methods:
        create(): Creates a new queue with the specified parameters.
        list(): Retrieves a list of queues.
    """

    def __init__(self, base_url: str, queue_id: str, read_timeout: int, max_batch: int):
        """
        Initializes a Queue object.

        Args:
            base_url (str): The base URL of the API.
            queue_id (str): The unique identifier for the queue.
            read_timeout (int): The time limit in seconds for which a message can be read from the queue.
            max_batch (int): The maximum number of messages that can be consumed in a batch.
        """
        assert type(base_url) == str
        assert type(queue_id) == str
        assert type(read_timeout) == int
        assert type(max_batch) == int
        self.base_url = base_url
        self.queue_id = queue_id
        self.read_timeout = read_timeout
        self.max_batch = max_batch

    def create(self) -> SuccessResponse:
        """
        create() -> ErrorResponse | SuccessResponse

        Creates a new queue with the specified parameters.

        Returns:
            SuccessResponse: An ErrorResponse exception if the request fails,
                or a SuccessResponse object if the request is successful.
        """
        r = requests.post(
            f"{self.base_url}/queue/new",
            json={
                "readTimeout": self.read_timeout,
                "maxBatch": self.max_batch,
                "queueId": self.queue_id,
            },
        )
        if r.status_code >= 400:
            raise ErrorResponse(f"failed to create queue. Response was {r.json()}")
        return SuccessResponse(r.json())

    def list(self):
        """
        list()

        Retrieves a list of queues.

        Returns:
            SuccessResponse: A SuccessResponse object containing the list of queues.
        """
        r = requests.get(f"{self.base_url}/queue/list")
        if r.status_code >= 400:
            raise ErrorResponse(f"failed to list queues. Response was {r.json()}")
        return SuccessResponse(r.json())


class Message:
    """
    Represents a message that can be produced to a queue or exchange.

    Attributes:
        message_id (str): The unique identifier for the message.
        content (str): The content of the message.
        queue (Queue): The Queue object associated with the message (optional).
        message_uuid (str): The UUID of the message (optional).

    Methods:
        set_uuid(uuid: str): Sets the UUID of the message.
        set_queue(queue: Queue): Sets the Queue object associated with the message.
        delete(): Deletes the message from the associated queue.
    """

    def __init__(self, message_id: str, content: str):
        assert type(message_id) == str
        assert type(content) == str
        self.queue = None
        self.message_uuid = None
        self.message_id = message_id
        self.content = content
        self.uuid = None

    def set_uuid(self, uuid: str):
        assert type(uuid) == str
        self.message_uuid = uuid

    def set_queue(self, queue: Queue):
        assert isinstance(queue, Queue)
        self.queue = queue

    def delete(self):
        """
        delete()

        Deletes the message from the associated queue as long as the message
            has been retrieved.

        Returns:
            SuccessResponse: A SuccessResponse object indicating a successful deletion.
        """
        assert self.queue is not None
        assert self.message_uuid is not None
        post_data = {"queueId": self.queue.queue_id, "messageUuid": self.message_uuid}
        r = requests.post(f"{self.queue.base_url}/message/delete", json=post_data)
        if r.status_code >= 400:
            raise ErrorResponse(
                f"failed to delete message {self.message_uuid}, {self.message_id}, {self.content}. Response was {r.json()}"
            )
        return SuccessResponse(r.json())


class MessageHandler:
    """
    Represents a message handler that can produce and consume messages from a queue.

    Attributes:
        base_url (str): The base URL of the API.
        queue (Queue): The Queue object associated with the message handler.

    Methods:
        produce(messages: list[Message]): Publishes a list of messages to the associated queue.
        consume(): Consumes messages from the associated queue.
    """

    def __init__(self, base_url: str, queue: Queue):
        """
        Initializes a MessageHandler object.

        Args:
            base_url (str): The base URL of the API.
            queue (Queue): The Queue object associated with the message handler.
        """
        assert type(base_url) == str
        assert isinstance(queue, Queue)
        self.base_url = base_url
        self.queue = queue

    def produce(self, messages: list[Message]):
        """
        produce(messages: list[Message])

        Publishes a list of messages to the associated queue.

        Args:
            messages (list[Message]): A list of Message objects to be published.
        """
        assert type(messages) == list
        for message in messages:
            assert isinstance(message, Message)

        messages = list(
            map(lambda m: {"messageId": m.message_id, "content": m.content}, messages)
        )
        r = requests.post(
            f"{self.base_url}/message/new",
            json={"messages": messages, "queueId": self.queue.queue_id},
        )
        if r.status_code >= 400:
            raise ErrorResponse(
                f"failed to publish messages {self.message_id}, {self.content}. Response was {r.json()}"
            )
        return SuccessResponse(r.json())

    def consume(self):
        """
        consume()

        Consumes messages from the associated queue.

        Returns:
            list[Message]: A list of Message objects received from the queue.
        """
        r = requests.get(f"{self.base_url}/message/get?queueId={self.queue.queue_id}")
        if r.status_code >= 400:
            raise ErrorResponse(
                f"failed to publish messages {self.message_id}, {self.content}. Response was {r.json()}"
            )

        messages_received = []
        for message in r.json()["data"]:
            message_obj = Message(message["messageId"], message["content"])
            message_obj.set_uuid(message["uuid"])
            message_obj.set_queue(self.queue)
            messages_received.append(message_obj)

        return messages_received


class Exchange:
    """
    Represents an exchange where messages can be routed to different queues.

    Attributes:
        base_url (str): The base URL of the API.
        queues (list[Queue]): The list of Queue objects bound to the exchange.
        exchange_id (str): The unique identifier for the exchange.
        exchange_type (str): The type of the exchange ("FANOUT" or "ID").

    Methods:
        create(): Creates a new exchange with the specified parameters.
        produce(messages: list[Message]): Publishes a list of messages to the associated exchange.
        list(): Retrieves a list of exchanges.
    """

    def __init__(
        self, base_url: str, queues: list[Queue], exchange_id: str, exchange_type: str
    ):
        """
        Initializes an Exchange object.

        Args:
            base_url (str): The base URL of the API.
            queues (list[Queue]): The list of Queue objects bound to the exchange.
            exchange_id (str): The unique identifier for the exchange.
            exchange_type (str): The type of the exchange ("FANOUT" or "ID").
        """
        assert type(base_url) == str
        assert type(queues) == list
        assert type(exchange_id) == str
        assert exchange_type == FANOUT or exchange_type == ID
        self.base_url = base_url
        self.queues = queues
        self.exchange_id = exchange_id
        self.exchange_type = exchange_type

    def create(self):
        """
        create()

        Creates a new exchange with the specified parameters.

        Returns:
            ErrorResponse : An ErrorResponse exception if the request fails,
                or a SuccessResponse object if the request is successful.
        """
        queue_ids = list(map(lambda q: q.queue_id, self.queues))
        r = requests.post(
            f"{self.base_url}/exchange/new",
            json={
                "id": self.exchange_id,
                "queueIds": queue_ids,
                "exchangeType": self.exchange_type,
            },
        )
        if r.status_code >= 400:
            raise ErrorResponse(
                f"failed to create exchange {self.exchange_id}. Response was {r.json()}"
            )
        return SuccessResponse(r.json())

    def list(self):
        """
        list()

        Retrieves a list of exchanges.

        Returns:
            SuccessResponse: A SuccessResponse object containing the list of exchanges.
        """
        r = requests.get(f"{self.base_url}/exchange/list")
        if r.status_code >= 400:
            raise ErrorResponse(f"failed to list exchanges. Response was {r.json()}")
        return SuccessResponse(r.json())

    def produce(self, messages: list[Message]):
        """
        produce(messages: list[Message])

        Publishes a list of messages to the associated exchange.

        Args:
            messages (list[Message]): A list of Message objects to be published.
        """
        assert type(messages) == list
        for message in messages:
            assert isinstance(message, Message)

        messages = list(
            map(lambda m: {"messageId": m.message_id, "content": m.content}, messages)
        )
        r = requests.post(
            f"{self.base_url}/exchange/add",
            json={"messages": messages, "exchangeId": self.exchange_id},
        )
        if r.status_code >= 400:
            raise ErrorResponse(
                f"failed to publish messages {self.message_id}, {self.content}. Response was {r.json()}"
            )
        return SuccessResponse(r.json())
