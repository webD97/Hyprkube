import { Gvk } from "../../model/k8s";

export interface ResourceMenuSimulatorProps {
    gvk: Gvk,
    namespace: string,
    name: string
}

// eslint-disable-next-line no-empty-pattern
export default function ResourceMenuSimulator({ }: ResourceMenuSimulatorProps) {
    return (
        <></>
    );
}
