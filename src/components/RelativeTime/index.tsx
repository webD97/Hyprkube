import dayjs from "dayjs";
import DurationPlugin, { Duration } from 'dayjs/plugin/duration';
import { useEffect, useState } from "react";

dayjs.extend(DurationPlugin);

function formatRelative(duration: Duration): string {
    const units = [
        { label: 'y', seconds: 365 * 24 * 60 * 60 },
        { label: 'd', seconds: 24 * 60 * 60 },
        { label: 'h', seconds: 60 * 60 },
        { label: 'm', seconds: 60 },
        { label: 's', seconds: 1 },
    ];

    let seconds = duration.asSeconds();
    const parts: string[] = [];

    for (const unit of units) {
        const value = Math.floor(seconds / unit.seconds);
        if (value > 0) {
            parts.push(`${value}${unit.label}`);
            seconds %= unit.seconds;
        }
    }

    return parts.length > 0 ? parts.slice(0, 2).join('') : '0s';
}

export interface RelativeTimeProps {
    timestamp: string
}

const RelativeTime: React.FC<RelativeTimeProps> = (props) => {
    const [, setGeneration] = useState(0);
    const now = dayjs();
    const diff = now.diff(dayjs(props.timestamp));
    const duration = dayjs.duration(diff);

    useEffect(() => {
        const interval = setInterval(() => {
            setGeneration(generation => generation + 1);
        });

        return () => clearInterval(interval);
    }, []);

    return (
        <>{formatRelative(duration)}</>
    );
};

export default RelativeTime;
