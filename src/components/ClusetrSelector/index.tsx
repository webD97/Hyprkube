import { KubeContextSource } from "../../hooks/useContextDiscovery";

export interface ClusterSelectorProps {
    contextSources: KubeContextSource[],
    selectedCluster?: KubeContextSource,
    onSelect: (contextSource: KubeContextSource) => void
}

export const ClusterSelector: React.FC<ClusterSelectorProps> = (props) => {
    const { contextSources, selectedCluster, onSelect } = props;

    return (
        <div style={{ display: 'flex', alignItems: 'center' }}>
            <label htmlFor="clusterSelector">Cluster:</label>
            <select id="clusterSelector" style={{ flexGrow: 1, marginLeft: '1em' }}
                value={contextSources.findIndex(cs => cs == selectedCluster)}
                onChange={(e) => onSelect(contextSources[parseInt(e.target.value)])}
            >
                <option disabled value={-1}></option>
                {
                    contextSources.map(([source, displayName], idx) => (
                        <option value={idx} key={`${displayName}@${source}`}>{displayName}</option>
                    ))
                }
            </select>
        </div>
    );
};
