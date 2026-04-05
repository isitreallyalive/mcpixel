from google.protobuf.message import Message

class Hashable[T: Message]:
    def __init__(self, proto: T):
        self.__proto = proto

    def get_proto(self) -> T:
        return self.__proto

    def __hash__(self):
        return hash(self.__proto.SerializeToString())

    def __eq__(self, other: Hashable):
        if not isinstance(other, Hashable):
            return False
        return self.__proto.SerializeToString() == other.__proto.SerializeToString()

    def __getattr__(self, name: str):
        return getattr(self.__proto, name)
