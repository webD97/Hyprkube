export type LogStreamEvent =
    | {
        event: 'newLine',
        data: {
            lines: string[]
        }
    }
    | {
        event: 'endOfStream'
    }
    | {
        event: 'error'
        data: {
            msg: string
        }
    };
