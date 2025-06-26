import dayjs from "dayjs";
import DurationPlugin, { Duration } from 'dayjs/plugin/duration';
import { useEffect, useState } from "react";

dayjs.extend(DurationPlugin);

function formatRelative(duration: Duration): string {
    if (duration.asSeconds() <= 0) return '0s';

    const years = duration.years();
    const days = duration.days();
    const hours = duration.hours();
    const minutes = duration.minutes();
    const seconds = duration.seconds();

    const result: string[] = [];

    if (years) result.push(`${years}y`);
    if (days) result.push(`${days}d`);
    if (hours) result.push(`${hours}h`);
    if (minutes) result.push(`${minutes}m`);
    if (seconds || result.length === 0) result.push(`${seconds}s`);

    return result.slice(0, 2).join('');
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
