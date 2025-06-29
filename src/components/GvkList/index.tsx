import React from 'react';
import { Gvk } from '../../model/k8s';

import { PhysicalPosition } from '@tauri-apps/api/dpi';
import { Menu } from '@tauri-apps/api/menu';
import classes from './component.module.css';

export interface GvkListProps {
    className?: string,
    gvks: Gvk[],
    flat?: boolean,
    withGroupNames?: boolean,
    onResourceClicked?: (gvk: Gvk) => void,
    onResourceAuxClicked?: (gvk: Gvk) => void,
    onGvkContextMenu: (gvk: Gvk) => Promise<Menu>,
}

const GvkList: React.FC<GvkListProps> = (props) => {
    const {
        className,
        gvks,
        flat = false,
        withGroupNames = false,
        onGvkContextMenu,
        onResourceClicked = () => undefined,
        onResourceAuxClicked = () => undefined,
    } = props;

    if (flat) {
        return (
            <div className={[classes.container, className].filter(c => !!c).join(' ')}>
                <ul>
                    {
                        gvks
                            .sort(({ group: groupA }, { group: groupB }) => groupA.localeCompare(groupB))
                            .map((gvk) => {
                                const { group, version, kind } = gvk;

                                return (
                                    <li key={`${kind}.${group}/${version}`}>
                                        <span
                                            onClick={() => onResourceClicked(gvk)}
                                            onAuxClick={(e) => e.button === 1 && onResourceAuxClicked(gvk)}
                                            onContextMenu={e => {
                                                e.preventDefault();
                                                onGvkContextMenu(gvk)
                                                    .then(menu => menu.popup(new PhysicalPosition(e.screenX, e.screenY)))
                                                    .catch(e => alert(JSON.stringify(e)));
                                            }
                                            }>
                                            {
                                                group !== '' && withGroupNames
                                                    ? <><span>{kind}&nbsp;</span><span className={classes.apiGroup}>({group})</span></>
                                                    : `${kind}`
                                            }
                                        </span>
                                    </li>
                                );
                            })
                    }
                </ul>
            </div>
        );
    }

    const groupedGvks: Record<string, Gvk[]> = {};

    gvks.forEach((gvk) => {
        const group = gvk.group;

        if (groupedGvks[group] === undefined) {
            groupedGvks[group] = [];
        }

        groupedGvks[group].push(gvk);
    });

    return (
        <div className={[classes.container, className].filter(c => !!c).join(' ')}>
            {
                Object.entries(groupedGvks)
                    .sort(([groupA], [groupB]) => groupA.localeCompare(groupB))
                    .map(([group, gvks], idx) => (
                        <details key={idx}>
                            <summary>
                                {
                                    (() => {
                                        if (group.length == 0) {
                                            return 'core';
                                        }

                                        const groupFragments = group.split('.');

                                        return <>
                                            <span>{groupFragments[0]}</span>
                                            {
                                                groupFragments.length > 1
                                                    ? <span className={classes.apiGroup}>.{groupFragments.slice(1).join('.')}</span>
                                                    : null
                                            }
                                        </>;
                                    })()
                                }
                            </summary>
                            <GvkList flat className={classes.indent}
                                gvks={gvks}
                                onResourceClicked={onResourceClicked}
                                onResourceAuxClicked={onResourceAuxClicked}
                                onGvkContextMenu={onGvkContextMenu}
                            />
                        </details>
                    ))
            }
        </div>
    );
};

export default GvkList;
