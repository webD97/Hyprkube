import ClusterCard, { Component } from "../../components/ClusterCard";

type Commander = { name: string, components: Component[] }

const commanders: Commander[] = [
    {
        name: 'Commander Stage', components: [
            { label: 'Grafana', status: 'warning' },
            { label: 'Prometheus', status: 'ok' },
            { label: 'Fleet', status: 'ok' },
        ]
    },
    {
        name: 'Commander Prod', components: [
            { label: 'Grafana', status: 'ok' },
            { label: 'Prometheus', status: 'ok' },
            { label: 'Fleet', status: 'ok' },
        ]
    },
];



const controllers: Commander[] = new Array(30).fill(0).map((_, idx) => ({
    name: `Controller Prod ${String(idx + 1).padStart(3, '0')}`, components: [
        { status: 'ok', label: 'Homeassistant' },
        { status: 'ok', label: 'PaperlessNGX' },
        { status: 'warning', label: 'PiHole' },
        { status: 'unknown', label: 'Jellyfin' },
        { status: 'error', label: 'Octoprint' },
    ]
}));

export const Playground: React.FC = () => {
    return (
        <div>
            <h1>Development playground</h1>
            <h2>ClusterCard</h2>

            <h3>Commanders</h3>
            <section style={{ display: "grid", gridTemplateColumns: "repeat(3, minmax(0, 1fr))", gap: "3em", maxWidth: '40%' }}>{
                commanders.map(({ name, components }) => (
                    <ClusterCard key={name} clusterName={name} clusterVersion="1.34.2+k3s1"
                        componentsStatus={components}
                        onSettingsClicked={() => undefined}
                    />
                ))
            }</section>

            <h3>Controllers</h3>
            <section style={{ display: "grid", gridTemplateColumns: "repeat(3, minmax(0, 1fr))", gap: "3em", maxWidth: '40%' }}>{
                controllers.map(({ name, components }) => (
                    <ClusterCard key={name} clusterName={name} clusterVersion="1.34.2+k3s1"
                        componentsStatus={components}
                        onSettingsClicked={() => undefined}
                    />
                ))
            }</section>
        </div>
    );
};