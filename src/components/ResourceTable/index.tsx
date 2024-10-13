import { GenericResource } from "../../model/k8s";
import dayjs from "dayjs";
import { JSONPath } from "jsonpath-plus";
import { ColumnDefinition, coreNodes, corePods, genericNamespacedResource, genericNonNamespacedResource } from "./columns";
import { CustomResourceColumnDefinition } from "kubernetes-types/apiextensions/v1";
import { useMemo } from "react";
import EmojiHint from "../EmojiHint";

export interface ResourceTableProps<R extends GenericResource> {
    resources: R[],
    onResourceClicked?: (resource: R) => void,
    additionalPrinterColumns?: CustomResourceColumnDefinition[]
}

function byCreationTimestamp(a: GenericResource, b: GenericResource) {
    const creationTimestampA = dayjs(a.metadata?.creationTimestamp);
    const creationTimestampB = dayjs(b.metadata?.creationTimestamp);

    return creationTimestampA.diff(creationTimestampB);
}

const ResourceTable = <R extends GenericResource>(props: ResourceTableProps<R>) => {
    const {
        resources,
        onResourceClicked = () => undefined,
        additionalPrinterColumns = []
    } = props;

    const columns = useMemo(() => {
        const { apiVersion, kind } = resources[0] || {};
        const isNamespaced = resources[0]?.metadata?.namespace || true;

        if (additionalPrinterColumns && additionalPrinterColumns.length > 0) {
            const defaultColumns = [...(isNamespaced ? genericNamespacedResource : genericNonNamespacedResource)];
            const ageColumn = defaultColumns[defaultColumns.length - 1];

            const additionalColumns: ColumnDefinition[] = additionalPrinterColumns.map(({ name, jsonPath, description }) => {
                return [name, { jsonPath: '$' + jsonPath, description }] as ColumnDefinition;
            });

            const mergedColumns = [...defaultColumns.slice(0, -1), ...additionalColumns];

            // If the additional columns don't bring their own age, we will do it
            if (mergedColumns[mergedColumns.length - 1][0] !== 'Age') {
                mergedColumns.push(ageColumn);
            }

            return mergedColumns;
        }

        if (apiVersion === 'v1' && kind === 'Pod') {
            return corePods;
        }
        if (apiVersion === 'v1' && kind === 'Node') {
            return coreNodes;
        }
        else if (isNamespaced) {
            return genericNamespacedResource;
        }
        return genericNonNamespacedResource;
    }, [resources, additionalPrinterColumns]);

    return (
        <>
            <table>
                <thead>
                    <tr>
                        {
                            columns.map(([name, config], idx) => (
                                <th key={`${name}@${idx}`} title={config.description}>
                                    {name}
                                    {config.description ? (<sup>💡</sup>) : null}
                                </th>
                            ))
                        }
                    </tr>
                </thead>
                <tbody>
                    {
                        resources.sort(byCreationTimestamp).reverse().map(resource => (
                            <tr key={resource.metadata?.uid} onClick={() => onResourceClicked(resource)}>
                                {
                                    columns.map(definition => definition[1]).map(({ jsonPath, transform = (value) => value }, idx) => (
                                        <td key={`${jsonPath}@${idx}`}>
                                            {
                                                (() => {
                                                    const value = transform(
                                                        JSONPath({
                                                            path: jsonPath,
                                                            json: resource
                                                        }));

                                                    if (typeof (value) === 'string') {
                                                        return (<span>{value}</span>);
                                                    }

                                                    return value;
                                                })()
                                            }
                                        </td>
                                    ))
                                }
                            </tr>
                        ))
                    }
                </tbody>
            </table>
            {
                resources.length == 0
                    ? <EmojiHint emoji="🤷‍♀️">No resource of this type found.</EmojiHint>
                    : null
            }
        </>
    );
}

export default ResourceTable;
