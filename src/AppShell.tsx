import { emit } from "@tauri-apps/api/event";
import { ErrorBoundary, FallbackProps } from "react-error-boundary";

import { useContext, useEffect, useRef } from "react";
import classes from './AppShell.module.css';
import MegaTabs, { MegaTabDefinition } from "./components/MegaTabs";
import StatusPanel from "./containers/StatusPanel";
import ApplicationTabsContext from "./contexts/ApplicationTabs";
import { useHeadlessTabs } from "./hooks/useHeadlessTabs";
import ClusterOverview from "./pages/ClusterOverview";

function fallbackRender(context: FallbackProps) {
    // Call resetErrorBoundary() to reset the error boundary and retry the render.

    return (
        <div role="alert">
            <p>Something went wrong:</p>
            <pre style={{ color: "red" }}>{JSON.stringify(context.error, undefined, 2)}</pre>
        </div>
    );
}

window.onbeforeunload = function () {
    void emit('frontend-onbeforeunload');
};

const Layout: React.FC = () => {
    const {
        applicationTabs,
        activeApplicationTab,
        pushApplicationTab,
        removeApplicationTab,
        setActiveApplicationTab,
    } = useContext(ApplicationTabsContext)!;

    const megaTabsOutlet = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (applicationTabs.length > 0) return;
        pushApplicationTab({ title: 'Connect to a cluster', icon: '🔮' }, () => <ClusterOverview />)
    }, [applicationTabs.length, pushApplicationTab]);

    return (
        <ErrorBoundary fallbackRender={fallbackRender}>
            <div className={classes.container}>
                <header className={classes.header}>
                    <MegaTabs
                        activeTab={activeApplicationTab}
                        setActiveTab={setActiveApplicationTab}
                        onCloseClicked={removeApplicationTab}
                        tabs={applicationTabs}
                        outlet={megaTabsOutlet}
                    />
                </header>
                <main className={classes.main} ref={megaTabsOutlet}>
                </main>
                <footer className={classes.footer}>
                    <StatusPanel />
                </footer>
            </div>
        </ErrorBoundary>
    );
};

const AppShell: React.FC = () => {
    const [
        applicationTabs,
        activeApplicationTab,
        pushApplicationTab,
        removeApplicationTab,
        setActiveApplicationTab,
        replaceApplicationTab
    ] = useHeadlessTabs<MegaTabDefinition>();

    return (
        <ApplicationTabsContext.Provider value={{ applicationTabs, activeApplicationTab, pushApplicationTab, removeApplicationTab, setActiveApplicationTab, replaceApplicationTab }}>
            <Layout />
        </ApplicationTabsContext.Provider>
    );
};

export default AppShell;
