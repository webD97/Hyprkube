import RelativeTime from "../../components/RelativeTime";
import { WindowControls } from "../../components/WindowControls";

export const Playground: React.FC = () => {
    return (
        <div>
            <h1>Development playground</h1>
            <h2>Window controls</h2>
            <WindowControls />
            <h2>RelativeTime</h2>
            <p>
                <RelativeTime timestamp="2024-06-20T12:00:00Z" />
            </p>
            <p>
                <RelativeTime timestamp={new Date().toISOString()} />
            </p>
        </div>
    );
};