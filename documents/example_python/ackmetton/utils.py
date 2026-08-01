import time

from queue import Queue, Full


# A class for measuring a delta time by stamping it with a start and end time and evaluating for the difference.
class Time_Stamp:
    def __init__(self, **kwargs):
        self.unit_mask = kwargs.get("unit_mask", 1000.0)

        self.start = 0
        self.end = 0
        self.delta = 0

        self.set_start()
        self.set_end()

    def set_start(self):
        self.start = time.time() * self.unit_mask
        return self.start

    def set_end(self):
        self.end = time.time() * self.unit_mask
        return self.end

    def set_delta(self):
        self.delta = self.end - self.start
        return self.delta

    def get_delta(self):
        self.set_end()
        return self.set_delta()


# A Custom Queue class that discards oldest elements.
class Discard_Oldest_Queue:
    def __init__(self, maxsize=3):
        self.queue = Queue(maxsize=maxsize)

    def put(self, item):
        try:
            self.queue.put(item, block=False)
        except Full:
            self.queue.get()
            self.queue.put(item, block=False)

    def get(self, block=True, timeout=None):
        return self.queue.get(block=block, timeout=timeout)

    def qsize(self):
        return self.queue.qsize()

    def empty(self):
        return self.queue.empty()

    def full(self):
        return self.queue.full()


class List_Queue:
    def __init__(self):
        self.queue = []

    def put(self, item):
        self.queue.append(item)

    def get(self):
        return self.queue.pop(0)
