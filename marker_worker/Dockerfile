FROM python:3.12-slim-bookworm

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    XDG_CACHE_HOME=/models \
    MARKER_MODE=fast \
    MARKER_WORKER_HOST=0.0.0.0 \
    MARKER_WORKER_PORT=8000

# opencv-python-headless (a Surya dependency) needs libglib; torch needs
# libgomp.  No display, CUDA or build toolchain is installed.
RUN apt-get update \
    && apt-get install -y --no-install-recommends libgomp1 libglib2.0-0 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 10001 worker \
    && useradd --system --create-home --uid 10001 --gid worker worker

# Install the pinned CPU runtime first.  The requirements file repeats these
# versions so a resolver cannot silently replace them with CUDA builds.
RUN pip install --index-url https://download.pytorch.org/whl/cpu \
    torch==2.7.0 torchvision==0.22.0

COPY requirements.txt /app/requirements.txt
RUN pip install -r /app/requirements.txt

COPY worker.py /app/worker.py
RUN mkdir -p /models && chown -R worker:worker /models /app

WORKDIR /app
USER 10001:10001
VOLUME ["/models"]
EXPOSE 8000

# The first startup downloads and loads the pinned models, so the start period
# is intentionally generous.  The endpoint reports 503 until models are ready.
HEALTHCHECK --interval=30s --timeout=10s --start-period=900s --retries=3 \
    CMD ["python", "-c", "import urllib.request; urllib.request.urlopen('http://127.0.0.1:8000/healthz', timeout=5)"]

CMD ["python", "/app/worker.py"]
