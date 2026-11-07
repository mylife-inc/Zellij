Table reference
===============

.. toctree::

   details

.. table:: **Feature** matrix

   +----------------+--------------------------------------------------------------------------------+
   | Feature        | Details                                                                        |
   +================+================================================================================+
   | Unicode        | Café 漢字 é                                                                    |
   +----------------+--------------------------------------------------------------------------------+
   | Links          | :doc:`Details <details>` and :ref:`install`                                    |
   +----------------+--------------------------------------------------------------------------------+
   | Downloads      | :download:`Notes <notes.txt>`                                                  |
   +----------------+--------------------------------------------------------------------------------+
   | Markup         | **Bold**, *emphasis*, ``a|{b}``, <script>literal</script>                      |
   +----------------+--------------------------------------------------------------------------------+
   | Paragraphs     | First paragraph.                                                               |
   |                |                                                                                |
   |                | Second paragraph.                                                              |
   +----------------+--------------------------------------------------------------------------------+
   | Blocks         | - First item                                                                   |
   |                | - Second item                                                                  |
   |                |                                                                                |
   |                | .. note:: Keep this.                                                           |
   |                |                                                                                |
   |                | .. code-block:: text                                                           |
   |                |                                                                                |
   |                |    +---+---+                                                                   |
   |                |    | a | b |                                                                   |
   |                |    +---+---+                                                                   |
   |                |    .. include:: /not/executed                                                  |
   +----------------+--------------------------------------------------------------------------------+
   | Image          | .. image:: /media/logo.svg                                                     |
   |                |    :alt: Table logo                                                            |
   +----------------+--------------------------------------------------------------------------------+

Simple data
-----------

========  ========
Group     Values
Key       Details
========  ========
one       First line extends beyond the border.
          Continuation with **strong** text.
--------  --------

..        Empty first cell.
three     First paragraph.

          Second paragraph.
========  ========

Headerless data
---------------

+--------------+--------------------------+
| Only data    | No invented header.      |
+--------------+--------------------------+

========  ========
first     Body only.
second    More body.
========  ========

.. note:: Nested table

   +----------+------------+
   | Inside   | A note     |
   +----------+------------+

List and CSV directives
-----------------------

.. list-table:: **List** matrix
   :header-rows: 1
   :stub-columns: 1
   :widths: auto

   * - Name
     - Value
   * - Alpha
     - List first paragraph.

       List second paragraph with :doc:`List details <details>`, :ref:`install`
       and :download:`List download <notes.txt>`.
   * - Beta
     - .. code-block:: text

          .. csv-table:: <script>{code}|literal</script>
   * - Gamma
     - .. image:: /media/logo.svg
          :alt: List logo
   * -
     - Empty list row heading.

.. csv-table:: CSV matrix
   :header: "Kind", "Text", "Tail"
            "Group", "Description", "Extra"
   :header-rows: 1
   :stub-columns: 1

   Label, Meaning, More
   Quotes,"Quoted ""word"", comma and **bold**.

   Second CSV paragraph.",end
   Download,:download:`CSV notes <notes.txt>`,saved
   Short,Padded CSV row

.. csv-table:: Unicode CSV
   :delim: │
   :quote: '

   Café│'漢字│quoted ''word'''
