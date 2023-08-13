import { useEffect, useRef, useState } from "react";
import Navbar from "./Navbar";
import { Piece, PracticeSession } from "./api-types";
import {
    addPiece,
    addPiecePracticed,
    addPracticeSession,
    fetchPieces,
    fetchPracticeSessions,
} from "./fetch";

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
    const [matchingPieces, setMatchingPieces] = useState<Piece[]>([]);

    useEffect(() => {
        fetchPieces(setPieces, alert);
    }, []);

    useEffect(() => {
        setMatchingPieces(
            pieces.filter(
                (piece) =>
                    piece.title.includes(title) &&
                    piece.composer.includes(composer)
            )
        );
    }, [composer, title, pieces]);

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
            { startDatetime, durationMins, instrument },
            (practice_session) => {
                for (let piece of piecesPracticed) {
                    addPiecePracticed(
                        {
                            pieceId: piece.piece_id,
                            practiceSessionId:
                                practice_session.practice_session_id,
                        },
                        () => {},
                        alert
                    );
                }
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
            <form onSubmit={handlePracticeSessionSubmit}>
                <input
                    type="datetime-local"
                    placeholder="start datetime"
                    value={startDatetime}
                    onChange={(e) => setStartDatetime(e.target.value)}
                />
                <input
                    type="number"
                    placeholder="duration (mins)"
                    value={durationMins}
                    onChange={(e) => setDurationMins(parseInt(e.target.value))}
                />
                <input
                    type="text"
                    placeholder="instrument"
                    value={instrument}
                    onChange={(e) => setInstrument(e.target.value)}
                />
                <input type="submit" />
            </form>
            <div>
                Pieces practiced:
                <br />
                {piecesPracticed.map((piece) => {
                    return (
                        <div key={piece.piece_id}>
                            {piece.composer}: {piece.title}
                        </div>
                    );
                })}
            </div>
            <button
                onClick={(e) => {
                    let dialog = piecesDialogRef.current;
                    if (dialog) {
                        dialog.showModal();
                    }
                }}
            >
                Edit pieces practiced
            </button>
            <dialog ref={piecesDialogRef}>
                <form onSubmit={handlePieceSubmit}>
                    <input
                        type="text"
                        value={composer}
                        onChange={(e) => setComposer(e.target.value)}
                        placeholder="composer"
                    />
                    <input
                        type="text"
                        value={title}
                        onChange={(e) => setTitle(e.target.value)}
                        placeholder="title"
                    />
                    <input type="submit" />
                </form>
                {matchingPieces.map((piece) => {
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

    return (
        <div>
            <Navbar />
            <AddPracticeSession setPracticeSessions={setPracticeSessions} />
            {practiceSessions.map((practiceSession) => {
                return (
                    <div>
                        <h4>
                            Practiced {practiceSession.instrument} for{" "}
                            {practiceSession.duration_mins} mins at{" "}
                            {practiceSession.start_datetime}
                        </h4>
                        <ul>
                            {practiceSession.pieces_practiced.map((piece) => {
                                return (
                                    <li>
                                        {piece.composer}: {piece.title}
                                    </li>
                                );
                            })}
                        </ul>
                    </div>
                );
            })}
        </div>
    );
}

export default PracticeSessions;
