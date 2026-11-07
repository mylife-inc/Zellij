Referenced examples
===================

Local code stays literal; explanatory prose remains searchable.

.. literalinclude:: snippets/Sample%20.py
   :language: text

Selected excerpt
----------------

.. literalinclude:: /snippets/Sample%20.py
   :start-after: BEGIN
   :end-before: END
   :lines: 2,1,2
   :dedent: 4
   :language: text

.. note:: Nested example

   .. literalinclude:: snippets/Sample%20.py
      :start-at: BEGIN
      :end-at: END
      :lines: 2
      :dedent:
      :language: text

- Included in a list.

  .. literalinclude:: snippets/Sample%20.py
     :lines: 3
     :dedent: 4
     :language: text

.. list-table:: Included in a cell

   * - .. literalinclude:: snippets/Sample%20.py
          :lines: 2-3
          :dedent:
          :language: text
