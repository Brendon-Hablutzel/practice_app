import React from "react";
import styles from "./css/CredentialForm.module.css";

interface Props {
    handleSubmit: (e: React.FormEvent) => void;
    userName: string;
    setUserName: (value: React.SetStateAction<string>) => void;
    password: string;
    setPassword: (value: React.SetStateAction<string>) => void;
}

function CredentialForm({
    handleSubmit,
    userName,
    setUserName,
    password,
    setPassword,
}: Props) {
    return (
        <div className={styles.formWrapper}>
            <div>
                <form onSubmit={handleSubmit}>
                    <div>
                        <input
                            type="text"
                            placeholder="Username"
                            value={userName}
                            onChange={(e) => setUserName(e.target.value)}
                            className={styles.textInput}
                        />
                        <br />
                        <input
                            type="password"
                            placeholder="Password"
                            value={password}
                            onChange={(e) => setPassword(e.target.value)}
                            className={styles.textInput}
                        />
                        <br />
                        <input type="submit" className={styles.submitButton} />
                    </div>
                </form>
            </div>
        </div>
    );
}

export default CredentialForm;
