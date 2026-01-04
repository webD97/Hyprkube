import ClusterCard from "../../components/ClusterCard";

export const Playground: React.FC = () => {
    return (
        <div>
            <h1>Development playground</h1>
            <h2>ClusterCard</h2>
            <section style={{ maxWidth: '600px' }}>
                <ClusterCard clusterName="Homelab" clusterVersion="v1.34.2+k3s1"
                    actions={[
                        { label: '📖 Connect', onTrigger: () => { } },
                    ]}
                />
            </section>
        </div>
    );
};