type Payload<K> = {
    repr: K,
}

export type WatchEvent<K> =
    | {
        event: 'created';
        data: Payload<K>
    }
    | {
        event: 'updated';
        data: Payload<K>
    }
    | {
        event: 'deleted';
        data: Payload<K>
    }