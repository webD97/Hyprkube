import { FcSynchronize } from "react-icons/fc";
import styles from './styles.module.css';

interface RotatingSpinnerProps {
    reverse?: boolean
}

const RotatingSpinner: React.FC<RotatingSpinnerProps> = ({ reverse = false }) => {
    return <div className={`${styles.container} ${reverse ? styles.reverse : ''}`}>
        <FcSynchronize />
    </div>
}

export default RotatingSpinner;
