interface Piece {
    composer: string;
    title: string;
    piece_id: number;
}

interface PracticeSession {
    start_datetime: string;
    duration_mins: number;
    instrument: string;
    pieces_practiced: Piece[];
    practice_session_id: number;
    user_id: number;
}

interface PiecePracticedMapping {
    practice_session_id: number;
    piece_id: number;
}

export { Piece, PracticeSession, PiecePracticedMapping };
