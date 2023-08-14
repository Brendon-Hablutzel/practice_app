import React, { useEffect, useRef, useState } from "react";
import Navbar from "./Navbar";
import { Piece, PracticeSession } from "./api-types";
import {
    addPiece,
    addPracticeSession,
    deletePracticeSession,
    fetchPieces,
    fetchPracticeSessions,
} from "./fetch";
import styles from "./css/PracticeSessions.module.css";

function AddPracticeSession({
    setPracticeSessions,
}: {
    setPracticeSessions: React.Dispatch<
        React.SetStateAction<PracticeSession[]>
    >;
}) {
    const [startDatetime, setStartDatetime] = useState("");
    const [durationMins, setDurationMins] = useState(0);
    const [instrument, setInstrument] = useState("");

    const [piecesPracticed, setPiecesPracticed] = useState<Piece[]>([]);

    const [title, setTitle] = useState("");
    const [composer, setComposer] = useState("");
    const piecesDialogRef = useRef<HTMLDialogElement>(null);

    const [pieces, setPieces] = useState<Piece[]>([]);

    useEffect(() => {
        let searchParams = {
            composer: composer !== "" ? composer : undefined,
            title: title !== "" ? title : undefined,
        };
        if (!(searchParams.composer || searchParams.title)) {
            setPieces([]);
        } else {
            fetchPieces(setPieces, alert, searchParams);
        }
    }, [title, composer]);

    const handlePracticeSessionSubmit = async (e: React.FormEvent) => {
        e.preventDefault();

        if (startDatetime === "") {
            alert("Invalid start datetime");
            return;
        }

        if (isNaN(durationMins) || durationMins <= 0) {
            alert("Invalid duration");
            return;
        }

        if (instrument === "") {
            alert("Invalid instrument");
            return;
        }

        addPracticeSession(
            { startDatetime, durationMins, instrument, piecesPracticed },
            (_) => {
                setStartDatetime("");
                setDurationMins(0);
                setInstrument("");
                setPiecesPracticed([]);
                fetchPracticeSessions(setPracticeSessions, alert);
            },
            alert
        );
    };

    const handlePieceSubmit = (e: React.FormEvent) => {
        e.preventDefault();

        if (composer === "") {
            alert("Invalid composer");
            return;
        }

        if (title === "") {
            alert("Invalid title");
            return;
        }

        addPiece(
            { composer, title },
            () => {
                setComposer("");
                setTitle("");
                fetchPieces(setPieces, alert);
            },
            alert
        );
    };

    return (
        <div>
            <h1>Search for or add practice sessions</h1>
            <form
                onSubmit={handlePracticeSessionSubmit}
                style={{ margin: "5px" }}
            >
                <input
                    type="datetime-local"
                    placeholder="Start datetime"
                    value={startDatetime}
                    onChange={(e) => setStartDatetime(e.target.value)}
                    className={styles.textInput}
                />
                <input
                    type="number"
                    placeholder="Duration (mins)"
                    value={durationMins}
                    onChange={(e) => setDurationMins(parseInt(e.target.value))}
                    className={styles.textInput}
                />
                <input
                    type="text"
                    placeholder="Instrument"
                    value={instrument}
                    onChange={(e) => setInstrument(e.target.value)}
                    className={styles.textInput}
                />
                <input
                    type="submit"
                    className={styles.submitButton}
                    value="Add Practice Session"
                />
            </form>
            <div className={styles.piecesPracticedContainer}>
                <div>
                    <div className={styles.piecesPracticedHeader}>
                        Pieces practiced:
                    </div>
                    <ul>
                        {piecesPracticed.map((piece) => {
                            return (
                                <li
                                    key={piece.piece_id}
                                    style={{ fontSize: "20px" }}
                                >
                                    {piece.composer}: {piece.title}
                                </li>
                            );
                        })}
                    </ul>
                </div>
                <button
                    onClick={(e) => {
                        let dialog = piecesDialogRef.current;
                        if (dialog) {
                            dialog.showModal();
                        }
                    }}
                    className={styles.editPiecesPracticedButton}
                >
                    Edit pieces practiced
                </button>
            </div>
            <dialog ref={piecesDialogRef}>
                <form onSubmit={handlePieceSubmit}>
                    <input
                        type="text"
                        value={composer}
                        onChange={(e) => setComposer(e.target.value)}
                        placeholder="Composer"
                        className={styles.textInput}
                    />
                    <input
                        type="text"
                        value={title}
                        onChange={(e) => setTitle(e.target.value)}
                        placeholder="Title"
                        className={styles.textInput}
                    />
                    <input
                        type="submit"
                        value="Add Piece"
                        className={styles.submitButton}
                    />
                </form>
                {pieces.map((piece) => {
                    if (piecesPracticed.includes(piece)) {
                        return (
                            <div key={piece.piece_id}>
                                {piece.composer}: {piece.title}
                                <button
                                    onClick={(e) => {
                                        setPiecesPracticed(
                                            piecesPracticed.filter(
                                                (piecePracticed) =>
                                                    piecePracticed.piece_id !==
                                                    piece.piece_id
                                            )
                                        );
                                    }}
                                    className={
                                        styles.removePiecePracticedButton
                                    }
                                >
                                    Remove
                                </button>
                            </div>
                        );
                    } else {
                        return (
                            <div key={piece.piece_id}>
                                {piece.composer}: {piece.title}
                                <button
                                    onClick={(e) => {
                                        setPiecesPracticed([
                                            ...piecesPracticed,
                                            piece,
                                        ]);
                                    }}
                                    className={styles.addPiecePracticedButton}
                                >
                                    Add
                                </button>
                            </div>
                        );
                    }
                })}
            </dialog>
        </div>
    );
}

function PracticeSessions() {
    const [practiceSessions, setPracticeSessions] = useState<PracticeSession[]>(
        []
    );

    useEffect(() => {
        fetchPracticeSessions(setPracticeSessions, alert);
    }, []);

    const handlePracticeSessionDelete = (practiceSession: PracticeSession) => {
        deletePracticeSession(
            practiceSession.practice_session_id,
            () => {
                fetchPracticeSessions(setPracticeSessions, alert);
            },
            alert
        );
    };

    return (
        <div>
            <Navbar />
            <AddPracticeSession setPracticeSessions={setPracticeSessions} />
            <div className={styles.practiceSessionsContainer}>
                <div style={{ width: "50%" }}>
                    {practiceSessions.map((practiceSession) => {
                        return (
                            <div className={styles.practiceSession}>
                                <div className={styles.practiceSessionHeader}>
                                    Practiced {practiceSession.instrument} for{" "}
                                    {practiceSession.duration_mins} mins at{" "}
                                    {practiceSession.start_datetime}
                                </div>
                                <ul>
                                    {practiceSession.pieces_practiced.map(
                                        (piece) => {
                                            return (
                                                <li key={piece.piece_id}>
                                                    {piece.composer}:{" "}
                                                    {piece.title}
                                                </li>
                                            );
                                        }
                                    )}
                                </ul>
                                <button
                                    onClick={(e) => {
                                        handlePracticeSessionDelete(
                                            practiceSession
                                        );
                                    }}
                                    className={
                                        styles.removePracticeSessionButton
                                    }
                                >
                                    Remove
                                </button>
                            </div>
                        );
                    })}
                </div>
            </div>
        </div>
    );
}

export default PracticeSessions;
